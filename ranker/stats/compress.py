#!/usr/bin/env python3


def compress(iterator, key):
    prev_row = None
    prev_row_has_changes = False

    for row in iterator:
        if prev_row is None:
            prev_row_has_changes = True
            yield row
        elif key(row) != key(prev_row):
            if not prev_row_has_changes:
                yield prev_row
            prev_row_has_changes = True
            yield row
        else:
            prev_row_has_changes = False
        prev_row = row

    if not prev_row_has_changes:
        yield prev_row


if __name__ == "__main__":
    import sys
    import csv

    reader = csv.reader(sys.stdin)
    writer = csv.writer(sys.stdout)
    for row in compress(reader, lambda row: row[1:]):
        writer.writerow(row)
        sys.stdout.flush()
