#!/usr/bin/fish
# This script will parse the Opcodes.json file, pulled from https://gbdev.io/gb-opcodes//optables/dark
# and will output the following information in the following format:
# opcode, "mnemonic", bytes, cycles
for op in (cat Opcodes.json | jq '.cbprefixed | map_values(keys) | keys | .[]')
    cat Opcodes.json | jq '.cbprefixed' | jq .$op | jq -j "$op"',", \"" ,.mnemonic,"\", ",.bytes,", ",.cycles[],"\n"'
end
