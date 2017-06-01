// src/cli.rs
// Argument parser and CLI (Command Line Interface)
// for BreakFast

use clap::{App, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("breakfastrs")
        .version("0.1")
        .about("BreakFast is a toolkit for detecting chromosomal rearrangements based on whole genome sequencing data.")
        .usage("\tbreakfast detect <bam_file> <genome> <out_prefix> [-a N] [-f N] [-q N] [-O OR] [--discard-duplicates=METHOD]\n\
        \tbreakfast detect specific [-A] <bam_file> <donors> <acceptors> <genome> <out_prefix>\n\
        \tbreakfast filter <sv_file> [-r P-S-A]... [--blacklist=PATH]\n\
        \tbreakfast annotate <sv_file> <bed_file>\n\
        \tbreakfast blacklist [--freq-above=FREQ] <sv_files>...\n\
        \tbreakfast visualize <sv_file>\n\
        \tbreakfast tabulate rearranged genes <sv_files>...\n\
        \tbreakfast tabulate fusions <sv_files>...\n\
        \tbreakfast statistics <sv_files>...\n\
        \tbreakfast filter by region <sv_file> <region>\n\
        \tbreakfast filter by distance <min_distance> <sv_file>\n\
        \tbreakfast align junction <reads>")

        .arg(Arg::with_name("anchor-len")
            .short("a")
            .long("anchor-len")
            .global(true)
            .takes_value(true)
            .value_name("N")
            .display_order(1)
            .help("Anchor length for split read analysis. When zero, split reads are not used [default: 0]"))

        .arg(Arg::with_name("max-frag-len")
            .short("f")
            .long("max-frag-len")
            .global(true)
            .takes_value(true)
            .value_name("N")
            .display_order(2)
            .help("Maximum fragment length [default: 5000]"))

        .arg(Arg::with_name("min-mapq")
            .short("q")
            .long("min-mapq")
            .global(true)
            .takes_value(true)
            .value_name("N")
            .display_order(3)
            .help("Minimum mapping quality to consider [default: 15]"))

        .arg(Arg::with_name("orientation")
            .short("O")
            .long("orientation")
            .global(true)
            .takes_value(true)
            .value_name("OR")
            .display_order(4)
            .help("Read pair orientation produced by sequencer. Either 'fr' (converging), 'rf' (diverging) or 'ff' [default: fr]"))

        .arg(Arg::with_name("all-reads")
            .short("A")
            .long("all-reads")
            .global(true)
            .display_order(5)
            .help("Use all reads for rearrangement detection, not just unaligned reads."))

        .arg(Arg::with_name("discard-duplicates")
            .long("discard-duplicates")
            .global(true)
            .takes_value(true)
            .value_name("Method")
            .display_order(6)
            .help("Method to use when discarding duplicate reads.'both-ends' considers a read pair (or unaligned read) to be a duplicate of another if the positions"))

        .arg(Arg::with_name("blacklist")
            .long("blacklist")
            .global(true)
            .takes_value(true)
            .value_name("list")
            .display_order(7)
            .help("Path to a file containing blacklisted regions."))

        .arg(Arg::with_name("min-reads")
            .short("r")
            .long("min-reads")
            .global(true)
            .takes_value(true)
            .value_name("P-S-A")
            .display_order(8)
            .help("Minimum number of spanning reads required to accept a breakpoint. Specified in the format P-S-A, where P=paired, S=split, A=either. For example, -r 1-2-0 would require at least one mate pair and two split reads of evidence [default: 0-0-0]."))

        .arg(Arg::with_name("freq-above")
            .long("freq-above")
            .global(true)
            .takes_value(true)
            .value_name("FREQ")
            .display_order(9)
            .help("Minimum frequency at which a variant must be present among the control samples to be considered a false positive [default: 0]."))

        .subcommand(SubCommand::with_name("detect")
            .usage("\tbreakfast detect <bam_file> <genome> <out_prefix> [-a N] [-f N] [-q N] [-O OR] [--discard-duplicates=METHOD]")
            .display_order(1)
            .arg(Arg::with_name("bam_file")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("genome")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("out_prefix")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("detect-specific")
            .usage("breakfast detect specific [-A] <bam_file> <donors> <acceptors> <genome> <out_prefix>")
            .display_order(2)
            .arg(Arg::with_name("bam_file")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("donors")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("acceptors")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("genome")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("out_prefix")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("filter")
            .usage("breakfast filter <sv_file> [-r P-S-A]... [--blacklist=PATH]")
            .display_order(3)
            .arg(Arg::with_name("sv_file")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("annotate")
            .usage("breakfast annotate <sv_file> <bed_file>")
            .display_order(4)
            .arg(Arg::with_name("sv_file")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("bed_file")
                .required(true)
                .takes_value(true)))


        .subcommand(SubCommand::with_name("blacklist")
            .usage("breakfast blacklist [--freq-above=FREQ] <sv_files>...")
            .display_order(5)
            .arg(Arg::with_name("sv_files")
                .required(true)
                .takes_value(true)))


        .subcommand(SubCommand::with_name("visualize")
            .usage("breakfast visualize <sv_file>")
            .display_order(6)
            .arg(Arg::with_name("sv_file")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("tabulate-rearranged-genes")
            .usage("breakfast tabulate rearranged genes <sv_files>...")
            .display_order(7)
            .arg(Arg::with_name("sv_files")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("tabulate-fusions")
            .usage("breakfast tabulate fusions <sv_files>...")
            .display_order(8)
            .arg(Arg::with_name("sv_files")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("statistics")
            .usage("breakfast statistics <sv_files>..")
            .display_order(9)
            .arg(Arg::with_name("sv_files")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("filter-by-region")
            .usage("breakfast filter by region <sv_file> <region>")
            .display_order(10)
            .arg(Arg::with_name("sv_file")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("region")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("filter-by-distance")
            .usage("breakfast filter by distance <min_distance> <sv_file>")
            .display_order(11)
            .arg(Arg::with_name("min_distance")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("sv_file")
                .required(true)
                .takes_value(true)))

        .subcommand(SubCommand::with_name("align-junction")
            .usage("breakfast align junction <reads>")
            .display_order(12)
            .arg(Arg::with_name("reads")
                .takes_value(true)
                .required(true)))


}
