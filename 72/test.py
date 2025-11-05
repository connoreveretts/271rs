#!/usr/bin/env python3

DEBUG = 0
CMD = "cargo run --"

import subprocess
import os
import random
from operator import add, sub, mul, floordiv as quo, mod as rem

# Generate large random numbers
bigone = random.randint(2 ** 500, 2 ** 512)
bigtwo = random.randint(2 ** 500, 2 ** 512)
hexone = hex(bigone)
hextwo = hex(bigtwo)

if DEBUG:
    print("\nhexone =\n", hexone, "\nhextwo = \n", hextwo)

# Test all operations
ops = {'ADD': add, 'SUB': sub, 'MUL': mul, 'QUO': quo, 'REM': rem}

for op in ops:
    try:
        result = int(subprocess.check_output(
            ["cargo", "run", "--", hexone, hextwo, op],
            stderr=subprocess.DEVNULL
        ), 16)
        
        answer = ops[op](bigone, bigtwo)
        
        if result != answer:
            print(f"Operator {op} failed.")
            if DEBUG:
                print("Expected:")
                print(hex(answer))
                print("Received:")
                print(hex(result))
            exit(1)
        else:
            print(f"{op} passes.")
    except subprocess.CalledProcessError as e:
        print(f"Operator {op} failed with error: {e}")
        exit(1)
    except ValueError as e:
        print(f"Operator {op} failed to parse result: {e}")
        exit(1)

print("\nAll tests passed!")
