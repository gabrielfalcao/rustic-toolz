#!/usr/bin/env bash

password='I X@X@ Nickelback <3'
plaintext="Hello World"
plaintext_filename="plain.txt"

if [ ! -e "${plaintext_filename}" ]; then
    echo -n "Hello World" > $plaintext_filename
fi

if [ -x ./target/release/aes-256-cbc ]; then
    bin="./target/release/aes-256-cbc";
else
    bin="./target/debug/aes-256-cbc";
fi
#auth=--key-filename "${key_filename}"
#auth=--password "${password}"

set -e
for i in $(seq 10); do
    export key_filename="key${i}.yaml";
    export cyphertext_filename="cyphertext${i}.yaml";
    ${bin} generate --key-filename "${key_filename}" --password "${password}";
done

for i in $(seq 10); do
    export key_filename="key${i}.yaml";
    export cyphertext_filename="cyphertext${i}.aes";
    ${bin} encrypt --key-filename "${key_filename}" --input-filename "${plaintext_filename}" --output-filename "${cyphertext_filename}"
    ${bin} decrypt --key-filename "${key_filename}" --input-filename "${cyphertext_filename}" --output-filename "${plaintext_filename}"
    test "$(cat $plaintext_filename)" == "${plaintext}"
done


# every key should decrypt every file since the key is derived
for i in $(seq 10); do
    for j in $(seq 10); do
        export key_filename="key${j}.yaml";
        export cyphertext_filename="cyphertext${i}.aes";
        echo "${key_filename}: ${cyphertext_filename}"
        ${bin} decrypt --key-filename "${key_filename}" --input-filename "${cyphertext_filename}" --output-filename "${plaintext_filename}"
        ${bin} decrypt --password "${password}" --input-filename "${cyphertext_filename}" --output-filename "${plaintext_filename}"
        test "$(cat $plaintext_filename)" == "${plaintext}"

    done
done
