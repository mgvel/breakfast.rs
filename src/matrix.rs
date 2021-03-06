
use crate::common::{parse_args, FileReader, read_bam_record};
use bitvec::*;
use rust_htslib::bam;
use rust_htslib::bam::Record;
use bio::alphabets::dna;
use rayon::prelude::*;

const USAGE: &str = "
Usage:
  breakfast matrix [options] <sv_file> <bam_files>...

Options:
  --threads=N         Maximum number of threads to use [default: 1]
";

// Each signature is 20+20 bp, covering both sides of the breakpoint,
// for a total of 40 bp.
#[derive(Debug)]
struct Rearrangement {
	signature: String,
	signature_revcomp: String,
	first_8_cols: String,
	//chromosome_left: String,
	//position_left: usize,
	//strand_left: char,
	//chromosome_right: String,
	//position_right: usize,
	//strand_right: char,
	//evidence: Vec<u32>
}

fn reverse_complement(seq: &str) -> String {
	String::from_utf8(dna::revcomp(seq.as_bytes())).unwrap()
}

//fn parse_strand(text: &str) -> char {
//	match text {
//		"+" => '+', "-" => '-',
//		_ => error!("Invalid strand '{}' found.", text)
//	}
//}

// This function takes the current hash as input, and modifies it according
// to the read sequence's next encoded base in the BAM file. The hash itself
// is the lower 16 bits of the u32. The upper 16 bits are used as error
// bits to indicate whether an ambiguous nucleotide was encountered.
// This way if the hash is larger than 65535, we know that the eight
// nucleotides used in the hash's calculation included some ambiguous ones.
fn hash_nucleotide(hash: u32, nuc: u8) -> u32 {
	((hash & 0b11111111_11111111_00111111_11111111u32) << 2) + match nuc {
		b'A' => 0u32, b'C' => 1u32, b'G' => 2u32, b'T' => 3u32,     // ACGT
		_ => 0b00000000_00000011_00000000_00000000u32   // Ambiguous nucleotide
	}
}

fn hash_8bp_sequence(seq: &str) -> u32 {
	assert!(seq.len() == 8);
	let mut hash: u32 = 0;
	for nuc in seq.bytes() { hash = hash_nucleotide(hash, nuc); }
	if hash & 0xFFFF0000u32 > 0 { error!("Invalid sequence '{}'.", seq); }
	hash
}

// Find the most frequent element in an unsorted vector. Operates in
// O(n log n) time.
fn most_frequent(elems: &Vec<String>) -> String {
	let mut sorted = elems.clone();
	sorted.sort_unstable();
	let mut most_frequent: usize = 0;
	let mut most_frequent_count: usize = 1;
	let mut curr_count: usize = 1;
	for k in 1..sorted.len() {
		if sorted[k] == sorted[k - 1] {
			curr_count += 1;
		} else {
			if curr_count > most_frequent_count {
				most_frequent = k - 1;
				most_frequent_count = curr_count;
			}
			curr_count = 1;
		}
	}
	if curr_count > most_frequent_count {
		most_frequent = sorted.len() - 1;
		most_frequent_count = curr_count;
	}
	sorted[most_frequent].clone()
}

fn count_rearrangements(bam_path: &str, rearrangements: &Vec<Rearrangement>)
	-> Vec<u32> {

	eprintln!("Analyzing {}...", bam_path);

	// Arrange junction signatures into a 65536-element table that is indexed
	// with the first 8 bp of the junction signature. This allows extremely
	// fast lookups.
	let mut signature_exists = bitvec![0; 65536];
	let mut signature_map: Vec<Vec<u32>> =
		(0..65536).map(|_| Vec::new()).collect();
	for r in 0..rearrangements.len() {
		// Add the signature and its reverse complement to the signature map.
		let hash = hash_8bp_sequence(&rearrangements[r].signature[16..24]);
		signature_exists.set(hash as usize, true);
		signature_map[hash as usize].push(r as u32);

		let hash = hash_8bp_sequence(
			&rearrangements[r].signature_revcomp[16..24]);
		signature_exists.set(hash as usize, true);
		signature_map[hash as usize].push(r as u32);
	}

	let mut supporting_reads = vec![0; rearrangements.len()];

	let mut bam = bam::Reader::from_path(&bam_path).unwrap_or_else(
		|_| error!("Could not open BAM file."));
	let mut read = Record::new();
	while read_bam_record(&mut bam, &mut read) {
		if read.is_unmapped() == false { continue; }
		if read.is_duplicate() { continue; }
		//if !count_aligned && read.is_unmapped() == false { continue; }
		//if !count_duplicates && read.is_duplicate() { continue; }

		let seq = String::from_utf8(read.seq().as_bytes()).unwrap();

		// Start with some error bits set, so we only start checking
		// against the signature map once we have hashed at least eight
		// nucleotides.
		let mut hash = 0b00000000_00000011_00000000_00000000u32;
		'outer: for base in seq.bytes() {
			hash = hash_nucleotide(hash, base);
			if hash & 0xFFFF0000u32 > 0 { continue; }
			if signature_exists[hash as usize] == false { continue; }
			for ridx in &signature_map[hash as usize] {
				// This read contains the 4+4 bp junction signature.
				// Now check if the 20+20 bp junction is also found.
				let rearrangement = &rearrangements[*ridx as usize];
				if seq.contains(&rearrangement.signature) ||
					seq.contains(&rearrangement.signature_revcomp) {
					supporting_reads[*ridx as usize] += 1;
					break 'outer;
				}
			}
		}
	}
	return supporting_reads
}

pub fn main() {
	let args = parse_args(USAGE);
	let sv_path = args.get_str("<sv_file>");
	let bam_paths = args.get_vec("<bam_files>");
	let threads: usize = args.get_str("--threads").parse().unwrap();
	//let count_duplicates = args.get_bool("--count-duplicates");
	//let count_aligned = args.get_bool("--count-aligned");

	// Convert BAM paths to sample names
	let mut samples: Vec<String> = Vec::new();
	for s in 0..bam_paths.len() {
		let start = if let Some(slash) =
			bam_paths[s].find('/') { slash + 1 } else { 0 };
		let end = if bam_paths[s].ends_with(".bam") {
			bam_paths[s].len() - 4 } else { bam_paths[s].len() };
		samples.push(bam_paths[s][start..end].into());
	}

	let mut line = String::new();
	let mut rearrangements: Vec<Rearrangement> = Vec::new();

	// Read all rearrangement signatures into memory
	let mut skipped_ambiguous = 0;
	let mut sv_file = FileReader::new(&sv_path);
	while sv_file.read_line(&mut line) {
		if line.starts_with("CHROM\t") { continue; }
		let mut signatures: Vec<String> = Vec::new();
		let cols: Vec<&str> = line.split('\t').collect();
		if cols.len() < 9 { continue; }
		let reads: Vec<&str> = cols[8].split(';').collect();
		for read in reads {
			let pipe = read.find('|').unwrap();
			if pipe < 20 { continue; }
			signatures.push(format!("{}{}",
				&read[pipe-20..pipe], &read[pipe+1..pipe+21]));
		}

		let mut signature = most_frequent(&signatures);
		signature.make_ascii_uppercase();
		if signature.chars().any(
			|b| b != 'A' && b != 'C' && b != 'G' && b != 'T') {
			eprintln!("WARNING: Skipping the following rearrangement because its consensus signature contains ambiguous nucleotides:\n{}", line);
			skipped_ambiguous += 1;
			continue;
		}
		let signature_revcomp = reverse_complement(&signature);

		let first_8_cols = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
			cols[0], cols[1], cols[2], cols[3], cols[4], cols[5],
			cols[6], cols[7]);

		rearrangements.push(Rearrangement {
			signature, signature_revcomp, first_8_cols
		});
	}
	if skipped_ambiguous > 0 {
		eprintln!("WARNING: Skipped {} rearrangements with signatures containing ambiguous nucleotides.", skipped_ambiguous);
	}

	rearrangements.sort_unstable_by(|a, b| a.signature.cmp(&b.signature));
	for k in 1..rearrangements.len() {
		if rearrangements[k - 1].signature == rearrangements[k].signature &&
			rearrangements[k - 1].first_8_cols !=
			rearrangements[k].first_8_cols {
			eprintln!("WARNING: Found two distinct rearrangements with same signature {}:\n{}\n{}\n",
				rearrangements[k].signature,
				rearrangements[k - 1].first_8_cols,
				rearrangements[k].first_8_cols);
		}
	}
	rearrangements.dedup_by(|a, b| a.signature == b.signature);

	eprintln!("Identifying supporting reads for {} rearrangements in {} BAM files...", rearrangements.len(), bam_paths.len());

	rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()
		.unwrap();
	let evidence: Vec<Vec<u32>> = bam_paths.par_iter()
		.map(|bam_path| count_rearrangements(&bam_path, &rearrangements))
		.collect();

	print!("CHROM\tSTRAND\tPOSITION\tNEARBY FEATURES\t");
	print!("CHROM\tSTRAND\tPOSITION\tNEARBY FEATURES\t");
	print!("SUPPORTING READS\tSIGNATURE\tNOTES");
	for sample in &samples { print!("\t{}", sample); }
	println!();
	for r in 0..rearrangements.len() {
		print!("{}", rearrangements[r].first_8_cols);
		print!("\t\t{}|{}\t", &rearrangements[r].signature[0..20], &rearrangements[r].signature[20..]);
		for e in &evidence { print!("\t{}", e[r]); }
		println!();
	}
}

