# Setup the tools needed to develop
setup-tools:
    cargo install cargo-release insta 
# publish rust crates    
release-crates:
    cargo release publish --no-verify --execute --no-confirm
