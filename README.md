# rpgmd
Rpg Maker Decrypter

# install

    cargo build
    cargo install --path ./

# run

    rpgmd <game_folder:optional> <output_directory:optional>

`game_folder` is the top level directory of the game. Default is the current directory.

`output_directory` is the folder all the decrypted files will be saved to. If this is not specified then the decrypted files are outputted next to the originals.
