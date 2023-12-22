import subprocess
import sys

import pandas as pd

if __name__ == "__main__":
    seed = int(sys.argv[1])
    file = f"{seed:04}"

    subprocess.run("cargo build --features local --release", shell=True)
    subprocess.run(
        "./target/release/ahc027" + f"< tools/in/{file}.txt > tools/out/{file}.txt",
        shell=True,
    )
    subprocess.run(
        "./tools/target/release/vis" + f" tools/in/{file}.txt tools/out/{file}.txt",
        shell=True,
    )
    subprocess.run(f"pbcopy < tools/out/{file}.txt", shell=True)

    df = pd.read_csv("./log/database.csv")
    print(df[(df.input_file == f"tools/in/{file}.txt")].sort_values("score"))
