cargo build --release
echo "15 characters." > 15char.txt
echo "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum." > lipsum.txt
curl https://github.com/cd-public/books/raw/main/pg1342.txt -o austen.txt 2>/dev/null
echo " === Finding errors vs. reference implementation. === "
diff <(sha512sum 15char.txt) <(./target/release/sha512 15char.txt)
diff <(sha512sum lipsum.txt) <(./target/release/sha512 lipsum.txt)
diff <(sha512sum austen.txt) <(./target/release/sha512 austen.txt)
rm 15char.txt lipsum.txt austen.txt
echo " === Errors printed. No errors denotes \"Perfect!\" === "
