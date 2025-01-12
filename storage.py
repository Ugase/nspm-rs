import os
from secrets import choice
from string import ascii_letters, digits, punctuation

import encrypt


def is_specialchar(char: str):
    return char in punctuation


def tps(password: str):
    """(T)est (P)assword (S)trength"""
    length = len(password)
    uppercase_lc = list(filter(str.isupper, password)).__len__()
    special_lc = list(filter(is_specialchar, password)).__len__()
    digits_lc = list(filter(str.isdigit, password)).__len__()
    uniquechars = list(set(password)).__len__()
    error = [1, 2, 3, 4, 5]
    if length < 14:
        error[0] = -1
    if uppercase_lc < 3:
        error[1] = -2
    if special_lc < 2:
        error[2] = -3
    if uniquechars < 6:
        error[3] = -4
    if digits_lc < 3:
        error[4] = -5
    return error


def cmp(master_password: str, file_name: str):
    """(C)reate (M)aster (P)assword"""
    salt = os.urandom(1024)
    with open(file_name, "wb") as file:
        file.write(encrypt.hash(master_password, salt))
    with open(f"{file_name}_salt", "wb") as msalt:
        msalt.write(salt)


def get_master_password(file_name: str):
    with open(file_name, "rb") as file:
        master_password = file.read()
    with open(f"{file_name}_salt", "rb") as salt:
        master_password_salt = salt.read()
    return master_password, master_password_salt


def init_dir(directory_name: str, master_password: str):
    os.mkdir(directory_name)
    os.mkdir(f"{directory_name}/salts")
    os.system(f"touch ./{directory_name}/passwords")
    os.system(f"touch ./{directory_name}/master_password")
    os.system(f"touch ./{directory_name}/services")
    cmp(master_password, f"./{directory_name}/master_password")


def verify_directory(directory_name: str):
    for i in [
        "salts",
        "passwords",
        "master_password",
        "master_password_salt",
        "services",
    ]:
        if i == "salts" and not os.path.isdir(os.path.join(directory_name, i)):
            return False
        elif not os.path.isfile(os.path.join(directory_name, i)) and i != "salts":
            return False
    return True


def get_salts(directory_name: str):
    salts: list[bytes] = []
    salt_amount = os.listdir(f"{directory_name}/salts").__len__()
    for i in range(salt_amount):
        with open(f"{directory_name}/salts/salt_{i}", "rb") as salt:
            salts.append(salt.read())
    return salts


def get_passwords(directory_name: str) -> list[bytes]:
    with open(f"{directory_name}/passwords", "rb") as passwords:
        p = passwords.read().split("\n".encode())
        del p[-1]
        return p


def merge(keys: list, values: list):
    return dict(zip(keys, values))


def decrypt_passwords(master_password: str, passwords: dict, salts: list[bytes]):
    decrypted_passwords = []
    for password, salt in zip(list(passwords.values()), salts):
        decrypted_passwords.append(encrypt.decrypt(master_password, password, salt))
    return merge(list(passwords.keys()), decrypted_passwords)


def encrypt_passwords(
    master_password: str,
    passwords: list[str],
    salt_loc: str,
):
    for password, salt_num in zip(passwords, range(passwords.__len__())):
        e = encrypt.encrypt(master_password, password, salt_loc + f"/salt_{salt_num}")
        yield e


def create_password(password: str, service: str, state: dict):
    return state | {service: password}


def save(state: dict, directory_location: str, master_password: str):
    with open(f"{directory_location}/passwords", "wb") as passwords:
        for passw in encrypt_passwords(
            master_password, list(state.values()), f"{directory_location}/salts"
        ):
            passwords.write(passw)
            passwords.write(b"\n")
    with open(f"{directory_location}/services", "w") as services:
        for service in list(state.keys()):
            services.write(service + "\n")
    return 0


def load(directory_name: str, master_password: str):
    services = open(f"{directory_name}/services").read().split("\n")
    passwords = get_passwords(directory_name)
    del services[-1]
    s = merge(services, passwords)
    return decrypt_passwords(master_password, s, get_salts(directory_name))


def generator(num_of_chr: int) -> str:
    """Generates a random list of numbers, letters and symbols"""
    sbol_list = digits + ascii_letters + punctuation
    result = "".join(choice(sbol_list) for _ in range(num_of_chr))
    return result
