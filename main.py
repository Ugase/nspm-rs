from os import get_terminal_size

import rich.table
from rich import print as printt

import storage
import ui
from encrypt import check_hash

print("Welcome to nspm!")
new = True
directory, master_password = ui.dir_selector(ui.propmt)
if not master_password:
    new = False
    master_password_hash = storage.gemp(f"{directory}/master_password")
    for _ in range(3):
        master = ui.password_prompt("Master password: ")
        if check_hash(master, master_password_hash):
            master_password = master
            break
    if not master_password:
        print("\033[1;31m3 incorrect password attempts\033[0m")
        quit(1)

if not new:
    state = storage.load(directory, master_password)
else:
    state = dict()
choices = [
    "1. List password",
    "2. Add password",
    "3. Edit password",
    "4. Remove password",
    "5. Generate password",
    "6. Save and quit",
]


def list_option():
    for option in choices:
        print(f"\033[3m{option}\033[0m")


while True:
    list_option()
    try:
        user = int(input("> "))
        if user > 6 or user <= 0:
            continue
    except:
        continue
    if user == 1:
        out = rich.table.Table(title="Passwords", show_lines=True, expand=True)
        out.add_column("Service")
        out.add_column("Password")
        for service, password in zip(list(state.keys()), list(state.values())):
            out.add_row(service, password)
        printt(out)
    if user == 2:
        service = input("Service: ")
        password = ui.new_password_prompt("Password: ")
        state = storage.create_password(password, service, state)
        print("Added pasword")
    if user == 3:
        index = input("Service: ")
        password = ui.password_prompt("Password: ")
        state[index] = password
        print("Successfully edited password")
    if user == 4:
        service = input("Service to remove: ")
        state.pop(service)
        print(f"Successfully removed {service}")
    if user == 5:
        length_of_password = input("Length of generated password (default 14): ")
        try:
            length_of_password = int(length_of_password)
            if length_of_password < 0:
                length_of_password = 14
        except:
            length_of_password = 14
        generated_password = storage.generator(length_of_password)
        print(f"Generated password: {generated_password}")
        confirmation = input("Do you want to save this password (y/n)? ")
        if confirmation.lower() in [
            "y",
            "yes",
            "ye",
            "yahoo",
            "yes i want to save this password to be able to access it again later",
            "save",
            "ok",
            "k",
            "sure",
            "fine",
            "finally",
            "just save",
            "es",
            "s",
            "se",
        ]:
            service = input("Service: ")
            state = storage.create_password(generated_password, service, state)
            print("Saved")
        else:
            continue
    if user == 6:
        print("Saving...")
        storage.save(state, directory, master_password)
        exit(0)
