#!/usr/bin/env python3

import sys

sum = 0
count = 0
for line in sys.stdin:
    sum += len(line)
    count += 1

if count != 0:
    print(sum / count)
else:
    print(0)
