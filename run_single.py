import ast
import subprocess
import sys

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

if __name__ == "__main__":
    plt.style.use("ggplot")
    seed = int(sys.argv[1])
    file = f"{seed:04}"

    subprocess.run("cargo build --features local --release", shell=True)
    subprocess.run(
        "./tools/target/release/tester ./target/release/ahc029"
        + f"< tools/in/{file}.txt > tools/out/{file}.txt",
        shell=True,
    )
    subprocess.run(
        "./tools/target/release/vis" + f" tools/in/{file}.txt tools/out/{file}.txt",
        shell=True,
    )
    subprocess.run(f"pbcopy < tools/out/{file}.txt", shell=True)

    # 過去ログとの比較
    df = pd.read_csv("./log/database.csv")
    print(df[(df.input_file == f"tools/in/{file}.txt")].sort_values("score"))

    # 詳細のビジュアライズ
    with open("./score.log", "r") as f:
        scores = ast.literal_eval(f.readline())
        invest_rounds = ast.literal_eval(f.readline())

    plt.plot(scores)
    for r in invest_rounds:
        plt.plot([r, r], [0, max(scores)], color="blue")
    plt.title(f"seed: {file}")
    plt.xlabel("round")
    plt.ylabel("score")
    plt.show()
