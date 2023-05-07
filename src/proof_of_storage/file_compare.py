FILE_A = "src/proof_of_storage/data/origin_data"
FILE_B = "src/proof_of_storage/data/unsealed_data"

SHOULD_PRINT_POS = False

is_right = True
wrong_count = 0
with open(FILE_A, "rb") as f_a:
    with open(FILE_B, "rb") as f_b:
        data_a = f_a.read()
        data_b = f_b.read()
        print(f'len: {len(data_a)} (origin) VS {len(data_b)} (unsealed)')
        for i in range(min(len(data_a), len(data_b))):
            if data_a[i] != data_b[i]:
                wrong_count += 1
                if SHOULD_PRINT_POS:
                    print(f'Something was wrong in: {i} / {i:x}!!!')
                is_right = False

if is_right:
    print("That's good! :)")
else:
    print("Oops, something was wrong! :(")
    print(f"Total wrong num is: {wrong_count}")