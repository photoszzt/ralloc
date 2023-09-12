import subprocess


def main():
    for test in [larson, prod_con, shbench, threadtest]:
        with open(f"{test.__name__}.csv", "a") as out:
            for allocator in ["cxlalloc", "r", "je"]:
                for threads in [1, 2, 4, 6, 10, 16, 20, 24, 32, 40]:
                    for trial in range(1):
                        out.write(f"{allocator},{threads},{test(allocator, threads)}\n")


def larson(allocator, threads):
    return run(
        [f"./{allocator}_larson_test", "30", "64", "400", "1000", "10000", "123", f"{threads}"],
        None,
        "Throughput",
        2,
    )


def prod_con(allocator, threads):
    return run(
        [f"./{allocator}_prod-con_test", f"{threads}", "10000000", "64"],
        None,
        "Time elapsed",
        3,
    )


def shbench(allocator, threads):
    return run(
        [f"./{allocator}_sh6bench_test"],
        f"100000\n64\n400\n{threads}\n",
        "rdtsc time",
        2,
    )


def threadtest(allocator, threads):
    return run(
        [f"./{allocator}_threadtest_test", f"{threads}", "10000", "100000", "0", "8"],
        None,
        "Time elapsed",
        3,
    )


def run(command, input, row, column):
    output = subprocess.run(command, input=input, stdout=subprocess.PIPE, check=True, text=True)
    for line in output.stdout.splitlines():
        if row in line:
            try:
                return int(line.split()[column])
            except ValueError:
                return float(line.split()[column])


if __name__ == "__main__":
    main()
