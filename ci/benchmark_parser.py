"""
benchmark_parser
----------------

Parse criterion benchmark or keys size results.
"""
import argparse
import csv
import pathlib
import json
import sys


ONE_HOUR_IN_NANOSECONDS = 3600E9

parser = argparse.ArgumentParser()
parser.add_argument('results',
                    help='Location of criterion benchmark results directory.'
                         'If the --key-size option is used, then the value would have to point to'
                         'a CSV file.')
parser.add_argument('output_file', help='File storing parsed results')
parser.add_argument('-d', '--database', dest='database',
                    help='Name of the database used to store results')
parser.add_argument('-w', '--hardware', dest='hardware',
                    help='Hardware reference used to perform benchmark')
parser.add_argument('-V', '--project-version', dest='project_version',
                    help='Commit hash reference')
parser.add_argument('-b', '--branch', dest='branch',
                    help='Git branch name on which benchmark was performed')
parser.add_argument('--commit-date', dest='commit_date',
                    help='Timestamp of commit hash used in project_version')
parser.add_argument('--bench-date', dest='bench_date',
                    help='Timestamp when benchmark was run')
parser.add_argument('--name-suffix', dest='name_suffix', default='',
                    help='Suffix to append to each of the result test names')
parser.add_argument('--append-results', dest='append_results', action='store_true',
                    help='Append parsed results to an existing file')
parser.add_argument('--walk-subdirs', dest='walk_subdirs', action='store_true',
                    help='Check for results in subdirectories')
parser.add_argument('--key-sizes', dest='key_sizes', action='store_true',
                    help='Parse only the results regarding keys size measurements')
parser.add_argument('--throughput', dest='throughput', action='store_true',
                    help='Compute and append number of operations per millisecond and'
                         'operations per dollar')


def recursive_parse(directory, walk_subdirs=False, name_suffix="", compute_throughput=False,
                    hardware_hourly_cost=None):
    """
    Parse all the benchmark results in a directory. It will attempt to parse all the files having a
    .json extension at the top-level of this directory.

    :param directory: path to directory that contains raw results as :class:`pathlib.Path`
    :param walk_subdirs: traverse results subdirectories if parameters changes for benchmark case.
    :param name_suffix: a :class:`str` suffix to apply to each test name found
    :param compute_throughput: compute number of operations per millisecond and operations per
        dollar
    :param hardware_hourly_cost: hourly cost of the hardware used in dollar

    :return: :class:`list` of data points
    """
    excluded_directories = ["child_generate", "fork", "parent_generate", "report"]
    result_values = []
    for dire in directory.iterdir():
        if dire.name in excluded_directories or not dire.is_dir():
            continue
        for subdir in dire.iterdir():
            if walk_subdirs:
                subdir = subdir.joinpath("new")
                if not subdir.exists():
                    continue
            elif subdir.name != "new":
                continue

            test_name = parse_benchmark_file(subdir)
            for stat_name, value in parse_estimate_file(subdir).items():
                test_name_parts = list(filter(None, [test_name, stat_name, name_suffix]))
                result_values.append({"value": value, "test": "_".join(test_name_parts)})

                if stat_name == "mean" and compute_throughput:
                    test_name_parts.append("ops-per-ms")
                    result_values.append({"value": compute_ops_per_millisecond(value),
                                          "test": "_".join(test_name_parts)})
                    test_name_parts.pop()

                    if hardware_hourly_cost is not None:
                        test_name_parts.append("ops-per-dollar")
                        result_values.append({
                            "value": compute_ops_per_dollar(value, hardware_hourly_cost),
                            "test": "_".join(test_name_parts)})

    return result_values


def parse_benchmark_file(directory):
    """
    Parse file containing details of the parameters used for a benchmark.

    :param directory: directory where a benchmark case results are located as :class:`pathlib.Path`

    :return: name of the test as :class:`str`
    """
    raw_res = _parse_file_to_json(directory, "benchmark.json")
    return raw_res["full_id"].replace(" ", "_")


def parse_estimate_file(directory):
    """
    Parse file containing timing results for a benchmark.

    :param directory: directory where a benchmark case results are located as :class:`pathlib.Path`

    :return: :class:`dict` of data points
    """
    raw_res = _parse_file_to_json(directory, "estimates.json")
    return {
        stat_name: raw_res[stat_name]["point_estimate"]
        for stat_name in ("mean", "std_dev")
    }


def parse_key_sizes(result_file):
    """
    Parse file containing key sizes results. The file must be formatted as CSV.

    :param result_file: results file as :class:`pathlib.Path`

    :return: :class:`list` of data points
    """
    result_values = []
    with result_file.open() as csv_file:
        reader = csv.reader(csv_file)
        for (test_name, value) in reader:
            result_values.append({"value": int(value), "test": test_name})

    return result_values


def compute_ops_per_dollar(data_point, product_hourly_cost):
    """
    Compute numbers of operations per dollar for a given ``data_point``.

    :param data_point: timing value measured during benchmark in nanoseconds
    :param product_hourly_cost: cost in dollar per hour of hardware used

    :return: number of operations per dollar
    """
    return ONE_HOUR_IN_NANOSECONDS / (product_hourly_cost * data_point)


def compute_ops_per_millisecond(data_point):
    """
    Compute numbers of operations per millisecond for a given ``data_point``.

    :param data_point: timing value measured during benchmark in nanoseconds

    :return: number of operations per millisecond
    """
    return 1E6 / data_point


def _parse_file_to_json(directory, filename):
    result_file = directory.joinpath(filename)
    return json.loads(result_file.read_text())


def dump_results(parsed_results, filename, input_args):
    """
    Dump parsed results formatted as JSON to file.

    :param parsed_results: :class:`list` of data points
    :param filename: filename for dump file as :class:`pathlib.Path`
    :param input_args: CLI input arguments
    """
    if input_args.append_results:
        parsed_content = json.loads(filename.read_text())
        parsed_content["points"].extend(parsed_results)
        filename.write_text(json.dumps(parsed_content))
    else:
        filename.parent.mkdir(parents=True, exist_ok=True)
        series = {
            "database": input_args.database,
            "hardware": input_args.hardware,
            "project_version": input_args.project_version,
            "branch": input_args.branch,
            "insert_date": input_args.bench_date,
            "commit_date": input_args.commit_date,
            "points": parsed_results,
        }
        filename.write_text(json.dumps(series))


def check_mandatory_args(input_args):
    """
    Check for availability of required input arguments, the program will exit if one of them is
    not present. If `append_results` flag is set, all the required arguments will be ignored.

    :param input_args: CLI input arguments
    """
    if input_args.append_results:
        return

    missing_args = []
    for arg_name in vars(input_args):
        if arg_name in ["results_dir", "output_file", "name_suffix",
                        "append_results", "walk_subdirs", "key_sizes",
                        "throughput"]:
            continue
        if not getattr(input_args, arg_name):
            missing_args.append(arg_name)

    if missing_args:
        for arg_name in missing_args:
            print(f"Missing required argument: --{arg_name.replace('_', '-')}")
        sys.exit(1)


if __name__ == "__main__":
    args = parser.parse_args()
    check_mandatory_args(args)

    raw_results = pathlib.Path(args.results)
    if not args.key_sizes:
        print("Parsing benchmark results... ")
        hardware_cost = None
        if args.throughput:
            print("Throughput computation enabled")
            ec2_costs = json.loads(
                pathlib.Path("ci/ec2_products_cost.json").read_text(encoding="utf-8"))
            try:
                hardware_cost = abs(ec2_costs[args.hardware])
                print(f"Hardware hourly cost: {hardware_cost} $/h")
            except KeyError:
                print(f"Cannot find hardware hourly cost for '{args.hardware}'")
                sys.exit(1)

        results = recursive_parse(raw_results, args.walk_subdirs, args.name_suffix, args.throughput,
                                  hardware_cost)
    else:
        print("Parsing key sizes results... ")
        results = parse_key_sizes(raw_results)
    print("Parsing results done")

    output_file = pathlib.Path(args.output_file)
    print(f"Dump parsed results into '{output_file.resolve()}' ... ", end="")
    dump_results(results, output_file, args)

    print("Done")
