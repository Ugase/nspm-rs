from base64 import urlsafe_b64encode
from hashlib import sha3_512 as sha512
from os import urandom

from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from rich.traceback import install

install(show_locals=True, extra_lines=50, code_width=150)


def hash(string: str, salt: bytes):
    final = string.encode() + salt
    return sha512(final).digest()


def check_hash(string: str, digest_hash: bytes, salt: bytes):
    return hash(string, salt) == digest_hash


def encrypt(master_password: str, password: str, salt_loc: str):
    salt = urandom(16)
    with open(salt_loc, "ab") as salting:
        salting.write(salt)
        salting.write("\n".encode())
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA3_512(),
        length=32,
        salt=salt,
        iterations=1_480_000,
    )
    key = urlsafe_b64encode(kdf.derive(master_password.encode()))
    f = Fernet(key)
    return f.encrypt(password.encode())


def decrypt(master_password: str, encrypted: bytes, salt: bytes):
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA3_512(), length=32, salt=salt, iterations=1_480_000
    )
    key = urlsafe_b64encode(kdf.derive(master_password.encode()))
    f = Fernet(key)
    return f.decrypt(encrypted).decode()
