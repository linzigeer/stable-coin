# 获取mapping账户数据（JSON格式）
solana account 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE -u d --output json --output-file pyth_mapping_devnet.json
# 获取价格账户数据（JSON格式）
solana account H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG -u d --output json --output-file pyth_price_account.json
solana program dump -u m rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ pyth_program.so