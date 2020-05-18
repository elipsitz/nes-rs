import subprocess

with open("roms/test/nestest.log") as f:
    expected = [x.strip() for x in f]

p = subprocess.run(
    ["cargo", "run", "roms/test/nestest.nes"],
    capture_output=True, text=True
)
actual = [x.strip() for x in p.stdout.split("\n") if ("_" * 10) in x]

print("expected: {} lines, actual: {} lines".format(len(expected), len(actual)))
for i in range(len(expected) - len(actual)):
    actual.append("")

for i in range(len(expected)):
    line_actual = actual[i]
    line_expected = expected[i]
    total_len = max(len(line_actual), len(line_expected))
    line_actual += " " * (total_len - len(line_actual))
    line_expected += " " * (total_len - len(line_expected))
    match = True
    for j in range(len(line_expected)):
        if (line_actual[j] != line_expected[j]) and (line_actual[j] != "_"):
            match = False

    if not match:
        print("EXPECTED:", expected[i])
        print("  ACTUAL:", actual[i])
        break