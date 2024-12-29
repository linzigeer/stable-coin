#solana-test-validator \
#    --bpf-program rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ pyth_program.so \
#    --account 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE pyth_mapping_devnet.json \
#    --account H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG pyth_price_account.json \
#    --bind-address 127.0.0.1 \
#    --rpc-port 8899 \
#    --limit-ledger-size \
#    -r
#solana-test-validator \
#    --url mainnet-beta \
#    --clone rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ \
#    --clone 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE \
#    --clone H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG \
#    --bind-address 127.0.0.1 \
#    --rpc-port 8899 \
#    --limit-ledger-size \
#    -r
solana-test-validator \
    --url mainnet-beta \
    --clone rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ \
    --clone 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE \
    --bind-address 127.0.0.1 \
    --rpc-port 8899 \
    --limit-ledger-size \
    -r