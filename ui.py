import readline
from os import chdir, getcwd, listdir, path

import getch

import storage

smcup = "\033[7\033[?47h"
rmcup = "\033[2J\033[?47l\0338"
cursor = "\033[H\033[2J\033[3J"
r = readline
help_message = """
There are a total of 7 commands (which have alaises):

choose (no other alias): Chooses a directory. Only accepts directories with the correct files
cd (no other alias): Changes current working directory
ls (no other alias): Lists the contents of the current working directory
exit (q, quit, ex): Exits the program
clear (c, cls): clears the screen
new (init, initialize, new_session, make): clears the screen and prompts the user for the new directories name and the master password to store the hash in the master_password file
help (h, ?): Shows this help

Usage:

Commands with no arguments: ls, exit, clear, help, new

cd: cd {dirname}
choose: choose {dirname}
"""


def dir_selector(string: str, dir_color="\033[1;94m"):
    """
    Directory selector, Only accepts directories as the arguments for choose (see the help by using the command ? for more info)
    args:
        string: the prompt that is displayed for the use subsitutes %P for the current working directory and %% for %
        dir_color: A SGR ansi escape code for the color of the directory when listing
    """
    print("\033[94mTip: Type ? for help!\033[0m")
    dire = path
    while True:
        strin = string.replace("%%", "%").replace("%P", getcwd())
        command_input = ""
        user = input(strin)
        hello = user.split()
        if not hello:
            continue
        command = hello[0]
        try:
            if command in ["choose", "cd"]:
                command_input = hello[1]
        except IndexError:
            print(
                "\033[1;31mThis command is supposed to have atleast 2 arguments (arguments are sperated by spaces)\033[0m"
            )
        if command == "choose":
            file_name = command_input
            if dire.isdir(file_name) and storage.verify_directory(file_name):
                return [f"{getcwd()}/{file_name}", ""]
            print(
                "\033[31;3mError: directory doesn't exist or is a file or the chosen directory doesn't have the sufficient files\033[0m"
            )
        elif command == "ls":
            for file in listdir():
                if dire.isdir(file):
                    print(dir_color + file + "\033[0m")
                    continue
                print(file)
        elif command == "cd":
            if dire.isdir(command_input) or command_input == "..":
                chdir(command_input)
            strin = string
        elif command.lower() in ["q", "quit", "exit", "ex"]:
            quit(0)
        elif command.lower() in ["c", "clear", "cls"]:
            print("\033[H\033[2J\033[3J")
        elif command.lower() in ["new", "init", "initialize", "make", "new_session"]:
            print(cursor)
            user_input = input("Directory name: ")
            master_password = new_password_prompt("Master password: ")
            storage.init_dir(user_input, master_password)
            return [user_input, master_password]
        elif command.lower() in ["help", "h", "?"]:
            print(help_message)


# Colors:
WARNING = "\033[3;33m"
DONE = "\033[3;32m"
NORMAL = "\033[0m"
DIRCOLOR = "\033[1;94m"

# Error dictionary
errors = {
    -1: "The length of the password should be 14 charecters long",
    -2: "The password should have atleast 3 capital letters",
    -3: "The password should have atleast 2 special charecters",
    -4: "The password should have atleast 6 unigue charecters",
    -5: "The password should have atleast 3 digits",
}

v = "✔ "
w = "⚠️ "
propmt = """\033[1;94m%P\033[0m\n\033[95m❯ \033[0m"""


def error_wrapper(message: int, error=False):
    if message > 0:
        message = message - (message * 2)
    if error:
        return w + WARNING + errors[message] + NORMAL
    return v + DONE + errors[message] + NORMAL


def warning_checker(output_of_tps: list[int]):
    for status in output_of_tps:
        yield error_wrapper(status, error=status < 0)


def new_password_prompt(pprompt: str, mask="*"):
    count = 0
    password_input = b""
    print(cursor)
    print(pprompt, flush=True, end="\n\n")
    for warning in warning_checker(storage.tps(password_input.decode())):
        print(warning)
    while True:
        char = getch.getch()
        if char == b"\x03":
            # Ctrl-C Character
            raise KeyboardInterrupt
        elif char == b"\x1b":
            # Escape character
            password_input = b""
            break
        elif char in ["\n", b"\r"]:
            break
        elif char in [b"\x08", b"\x7f", ""]:
            if count != 0:
                print(cursor)
                print("\b \b" * len(mask), end="", flush=True)
                count -= 1
                password_input = password_input[:-1]
                print(pprompt, end="")
                print((mask * count), end="\n\n", flush=True)
                for warning in warning_checker(storage.tps(password_input.decode())):
                    print(warning)
        else:
            print(cursor)
            count += 1
            password_input += char.encode()
            print(pprompt, end="")
            print((mask * count), end="\n\n", flush=True)
            for warning in warning_checker(storage.tps(password_input.decode())):
                print(f"{warning}")
    password = password_input.decode()
    print(cursor)
    return password


def password_prompt(pprompt: str, mask="*"):
    count = 0
    password_input = b""
    print(cursor)
    print(pprompt, flush=True, end="")
    while True:
        char = getch.getch()
        if char == b"\x03":
            # Ctrl-C Character
            raise KeyboardInterrupt
        elif char == b"\x1b":
            # Escape character
            password_input = b""
            break
        elif char in ["\n", b"\r"]:
            break
        elif char in [b"\x08", b"\x7f", ""]:
            if count != 0:
                print(cursor)
                print("\b \b" * len(mask), end="", flush=True)
                count -= 1
                password_input = password_input[:-1]
                print(pprompt, end="")
                print((mask * count), end="", flush=True)
        else:
            print(cursor)
            count += 1
            password_input += char.encode()
            print(pprompt, end="")
            print((mask * count), end="", flush=True)
    password = password_input.decode()
    print(cursor)
    return password
