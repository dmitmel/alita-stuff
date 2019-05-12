#!/usr/bin/env python3

import sys
import csv


reader = csv.reader(sys.stdin)
writer = csv.writer(sys.stdout)

prev_row = None
prev_row_has_change = False

for row in reader:
    if prev_row is None:
        writer.writerow(row)
        prev_row_has_change = True
    elif row[1:] != prev_row[1:]:
        if not prev_row_has_change:
            writer.writerow(prev_row)
        writer.writerow(row)
        sys.stdout.flush()
        prev_row_has_change = True
    else:
        prev_row_has_change = False
    prev_row = row

if not prev_row_has_change:
    writer.writerow(prev_row)
