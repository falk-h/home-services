#!/usr/bin/env python3
import math
import random

LEN = 48
COLOR = "7e9cd8"
SIGMA = 1
ALPHA = "05"

HCOUNT = 50
HCOLOR = "97ba6b"
HALPHA_MIN = 10
HALPHA_MAX = 20
HEXP = 3
HBLUR = 10

print(',\n'.join([f"h{x}" for x in range(1, 7)]), end="")
print(" {")
print("    text-shadow:")

shadows = []
# for i in range(256):
#     offset = i * LEN / 256
#     count = random.randrange(i//5+1)
#     for _ in range(count):
#         y = -offset * random.normalvariate(0, SIGMA) * LEN
#         x = offset * random.normalvariate(0, SIGMA) * LEN
#         shadows+=[f"        {x:>10}px {y:>10}px 0 #{COLOR}{ALPHA}"]

for i in range(1, HCOUNT + 1):
    theta = math.pi / 2 * i / HCOUNT
    k = math.sin(theta) ** HEXP
    alpha = round(k * (HALPHA_MAX - HALPHA_MIN) + HALPHA_MIN)
    blur = HBLUR * (1 - k)
    y = round(i / HCOUNT, 2)
    y = str(y)
    shadows+=[f"0 {y.lstrip('0')}em {blur}px #{HCOLOR}{alpha:02x}"]
    shadows+=[f"0 -{y}em {blur}px #{HCOLOR}{alpha:02x}"]

print(",\n".join(shadows), end="")
print(";")
print("}")

# print("p::first-line {")
#
# shadows = []
# for i in range(256):
#     offset = i * LEN / 256
#     count = random.randrange(i//5+1)
#     for _ in range(count):
#         y = -offset * random.normalvariate(0, SIGMA) * LEN
#         x = offset * random.normalvariate(0, SIGMA) * LEN
#         shadows+=[f"        {x:>10}px {y:>10}px 0 #{COLOR}{ALPHA}"]
#
# print("    text-shadow:")
# print(",\n".join(shadows), end="")
# print(";")
# print("}")
