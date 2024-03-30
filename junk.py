"""
Generate junk.
"""


JUNK = [
    "",
    ";",
    "3;",
    "();",
    "{;};",
    "({});",
    "{();};",
    "*&*&();",
    "((),());",
    "let _=();",
    "if true{};",
    "let _=||();",
    "loop{break};",
    "loop{break;};",
    "if let _=(){};",
]

gen = []
for i in range(81):
    if i < len(JUNK):
        gen.append(JUNK[i])
    else:
        half = i // 2
        rest = i - half
        gen.append(f"{gen[half]}{gen[rest]}")

print(f"const JUNK: [&str; {len(gen)}] = [")
for junk in gen:
    print(f'    "{junk}",')
print("];")
