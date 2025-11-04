#!/usr/bin/env python3
"""
Advanced condense for CPU instruction traces — fixed numeric-stream bug.
Input:  result.log (or result_condensed.log)
Output: result_condensed_more.log
        loop_templates.txt  (legend & details)
"""

import re
from collections import defaultdict, Counter
import hashlib

INPUT = "result.log"           # change to your file (e.g. result_condensed.log)
OUTPUT = "result_condensed_more.log"
LEGEND = "loop_templates.txt"

# ------------ helpers ------------
_RE_MULTI_SP = re.compile(r"\s{2,}")
_RE_HEX_NUM = re.compile(r'(?i)(#?\$?[0-9A-F]+)')  # catches $FF, #$01, 90D4, etc.

def split_columns(line):
    parts = _RE_MULTI_SP.split(line.strip())
    return parts

def extract_instruction_text(line):
    parts = split_columns(line)
    for p in parts:
        if re.search(r'[A-Za-z]{2,}', p):
            return p.strip()
    return line.strip()

def make_template(instr_text):
    nums = []
    def _repl(m):
        tok = m.group(1)
        val = tok.lstrip('#$')
        try:
            num = int(val, 16)
        except:
            try:
                num = int(val)
            except:
                num = None
        if num is not None:
            idx = len(nums)
            nums.append(num)
            return "{N%d}" % idx
        else:
            return tok
    template = _RE_HEX_NUM.sub(_repl, instr_text)
    tokens = template.split()
    return template, tuple(nums), tokens

# ------------ read & parse ------------
print("Reading lines...")
with open(INPUT, "r", errors="ignore") as f:
    raw_lines = [l.rstrip("\n") for l in f if l.strip()]

entries = []
for i, line in enumerate(raw_lines):
    instr = extract_instruction_text(line)
    templ, nums, tokens = make_template(instr)
    entries.append({
        "orig": line,
        "instr": instr,
        "template": templ,
        "tokens": tokens,
        "nums": nums,   # ALWAYS a tuple (possibly empty)
        "idx": i
    })

print(f"Parsed {len(entries)} instruction lines.")

# ------------ detect consecutive template repeats (block compression) ------------
print("Detecting consecutive repeated templates (with numeric sequences)...")
i = 0
compressed = []
while i < len(entries):
    max_blk = 12
    found = False
    for blk_size in range(max_blk, 0, -1):
        if i + blk_size*2 > len(entries):
            continue
        # Check template equality for the block
        block_templates = tuple(e["template"] for e in entries[i:i+blk_size])
        count = 1
        j = i + blk_size
        # Initialize numeric_streams as list of lists-of-tuples (one list per line in block)
        numeric_streams = [[e["nums"]] for e in entries[i:i+blk_size]]
        while j + blk_size <= len(entries) and tuple(e["template"] for e in entries[j:j+blk_size]) == block_templates:
            for k in range(blk_size):
                # append the tuple of nums for that instance
                numeric_streams[k].append(entries[j+k]["nums"])
            count += 1
            j += blk_size
        if count > 1:
            # Analyze numeric streams for arithmetic progression
            progression_info = []
            for k, numseqs in enumerate(numeric_streams):
                # numseqs is a list of tuples; e.g. [(a0,a1...), (b0,b1...), ...]
                if not numseqs or not numseqs[0]:
                    progression_info.append(None)
                    continue
                num_placeholders = len(numseqs[0])
                per_placeholder = []
                for p in range(num_placeholders):
                    seq = []
                    valid = True
                    for ns in numseqs:
                        if p < len(ns):
                            seq.append(ns[p])
                        else:
                            valid = False
                            break
                    if (not valid) or len(seq) < 2:
                        per_placeholder.append(None)
                        continue
                    diffs = [seq[t+1]-seq[t] for t in range(len(seq)-1)]
                    if all(d == diffs[0] for d in diffs):
                        per_placeholder.append((seq[0], seq[-1], diffs[0]))
                    else:
                        per_placeholder.append(None)
                progression_info.append(per_placeholder)
            compressed.append({
                "type": "block",
                "start": i,
                "blk_size": blk_size,
                "count": count,
                "templates": block_templates,
                "sample_lines": [entries[x]["orig"] for x in range(i, i+blk_size)],
                "progressions": progression_info
            })
            i = j
            found = True
            break
    if not found:
        compressed.append({"type": "line", "line": entries[i]["orig"], "idx": i})
        i += 1

print("Finished detecting consecutive repeated template blocks.")

# ------------ frequency-driven global substitution ------------
print("Building global frequency map of block templates...")
freq = Counter()
for c in compressed:
    if c["type"] == "block":
        key = "||".join(c["templates"])
        freq[key] += c["count"]

TOP_N = 120
top_templates = [k for k, _ in freq.most_common(TOP_N)]
template_tag = {k: f"<L{i+1:03d}>" for i, k in enumerate(top_templates)}

# ------------ produce final output with legend ------------
print("Producing final output...")
with open(OUTPUT, "w") as out, open(LEGEND, "w") as leg:
    leg.write("Legend of loop templates (automatically generated)\n\n")
    for c in compressed:
        if c["type"] == "line":
            out.write(c["line"] + "\n")
        else:
            key = "||".join(c["templates"])
            tag = template_tag.get(key)
            if tag:
                out.write(f"{tag} ×{c['count']}\n")
                out.write("# sample:\n")
                for s in c["sample_lines"]:
                    out.write("# " + s + "\n")
                leg.write(f"{tag}  occurrences: {freq[key]}  block_size:{c['blk_size']}  repeated_instances:{c['count']}\n")
                leg.write("sample block:\n")
                for s in c["sample_lines"]:
                    leg.write(s + "\n")
                leg.write("\n")
            else:
                prog_summ = []
                for line_p, pinfo in enumerate(c["progressions"]):
                    if not pinfo:
                        prog_summ.append(None)
                        continue
                    ph = []
                    for phinfo in pinfo:
                        if phinfo:
                            ph.append(f"{hex(phinfo[0])}->{hex(phinfo[1])},step={hex(phinfo[2])}")
                        else:
                            ph.append(None)
                    prog_summ.append(ph)
                out.write(f"[Block@{c['start']} x{c['count']} size={c['blk_size']}]")
                if any(prog_summ):
                    out.write(" prog=" + str(prog_summ))
                out.write("\n")
                for s in c["sample_lines"]:
                    out.write(s + "\n")
    leg.write("\nTop Template Tags:\n")
    for k, t in template_tag.items():
        leg.write(f"{t} occurrences: {freq[k]} template:\n")
        for templ_part in k.split("||"):
            leg.write(templ_part + "\n")
        leg.write("\n")

print("Done.")
print("Outputs:", OUTPUT, LEGEND)
