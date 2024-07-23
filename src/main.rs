fn main() {
    for arg in std::env::args().skip(1) {
        unicop::check_file(&arg);
    }
}
