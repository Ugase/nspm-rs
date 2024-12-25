from os import get_terminal_size

import storage
import ui
from encrypt import check_hash

terminal = get_terminal_size()

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
    "5. Save and quit",
]


def list_option():
    for option in choices:
        print(f"\033[3m{option}\033[0m")


while True:
    list_option()
    try:
        user = int(input("> "))
        if user > 5 or user <= 0:
            continue
    except:
        continue
    if user == 1:
        print("Service" + " " * (terminal.columns - 15) + "Passwords")
        for service, password in zip(list(state.values()), list(state.keys())):
            space = " " * (terminal.columns - (len(service) + len(password)))
            print(service + space + password)
    if user == 2:
        service = input("Service: ")
        password = ui.new_password_prompt("Password: ")
        state = storage.create_password(service, password, state)
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
        print("Saving...")
        storage.save(state, directory, master_password)
        exit(0)
