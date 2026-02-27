import sys

def remove_every_other_line(input_path, output_path):
    with open(input_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    kept_lines = lines[::2]  # Keep lines 0, 2, 4, ...

    with open(output_path, 'w', encoding='utf-8') as f:
        f.writelines(kept_lines)

    print(f"Kept {len(kept_lines)} of {len(lines)} lines -> {output_path}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python remove_every_other_line.py <input_file> <output_file>")
        sys.exit(1)

    remove_every_other_line(sys.argv[1], sys.argv[2])
