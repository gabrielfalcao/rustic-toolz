#!/usr/bin/env bash

password='I X@X@ Nickelback <3'
plaintext="Hello World"
bin="./target/debug/aes256"
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
    ${bin} encrypt --password "${password}" --cyphertext-filename "${cyphertext_filename}" --input-filename Cargo.toml;
    # ${bin} encrypt --password "${password}" --cyphertext-filename "${cyphertext_filename}" --string "${plaintext}";

    ${bin} decrypt --key-filename "${key_filename}" --cyphertext-filename "${cyphertext_filename}";
done


# every key should decrypt every file since the key is derived
for i in $(seq 10); do
    for j in $(seq 10); do
        export key_filename="key${j}.yaml";
        export cyphertext_filename="cyphertext${i}.aes";
        echo "${key_filename}: ${cyphertext_filename}"
        ${bin} decrypt --key-filename "${key_filename}" --cyphertext-filename "${cyphertext_filename}";
        ${bin} decrypt --password "${password}" --cyphertext-filename "${cyphertext_filename}";
    done
done
