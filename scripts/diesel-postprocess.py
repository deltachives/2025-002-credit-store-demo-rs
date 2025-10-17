import os
from typing import List, Tuple
from dataclasses import dataclass
import checkpipe as pipe

file_path = os.path.realpath(__file__)
project_path = os.path.join(os.path.dirname(file_path), "..")


@dataclass
class Rule:
    activator: str
    target: str
    replacement: str


@dataclass
class RuleWithStatus:
    rule: Rule
    active: bool
    spent: bool


def make_rules() -> List[Rule]:

    with open(os.path.join(project_path, "scripts", "schema.rs.replace"), "r") as f1:
        lines = f1.readlines()

    lines_filtered = (
        lines.__iter__()
        | pipe.OfIter[str].filter(lambda line: line.strip() != "" and not line.strip().startswith("//"))
        | pipe.OfIter[str].to_list()
    )

    if len(lines_filtered) == 0 or len(lines_filtered) % 3 != 0:
        raise Exception("schema.rs.replace Must be composed of triplets")

    mut_results = []

    for i in range(len(lines_filtered) // 3):
        activator = lines_filtered[3*i].strip()
        target = lines_filtered[3*i + 1].strip()
        replacement = lines_filtered[3*i + 2].strip()

        mut_results.append(Rule(activator, target, replacement))

    return mut_results


def postprocess(rules: List[Tuple[str, str]]):
    """
    Each rule should be processed once. When a rule is active, make a replace with its target.
    """

    mut_rules_with_status: List[RuleWithStatus] = (
        rules.__iter__()
        | pipe.OfIter[Rule].map(lambda rule: RuleWithStatus(rule, False, False))
        | pipe.OfIter[RuleWithStatus].to_list()
    )

    def get_new_line(line: str) -> str:
        for rule in mut_rules_with_status:
            if not rule.spent:
                if rule.active:
                    if rule.rule.target in line:
                        rule.spent = True
                        return line.replace(rule.rule.target, rule.rule.replacement)
                else:
                    if rule.rule.activator in line:
                        rule.active = True

        return line

    with open(os.path.join(project_path, "src", "autogen", "schema.rs"), "r") as f:
        lines = f.readlines()

    with open(os.path.join(project_path, "src", "autogen", "schema.rs"), "w") as f:
        with open(os.path.join(project_path, "scripts", "schema.rs.pre"), "r") as f2:
            for line in f2.readlines():
                f.write(line)

        f.write("\n\n")

        for line in lines:
            f.write(get_new_line(line))

    for rule in mut_rules_with_status:
        if not rule.spent:
            raise Exception(f'Rule "{rule.rule.activator}" "{rule.rule.target}" has not been activated')


if __name__ == "__main__":
    postprocess(make_rules())
