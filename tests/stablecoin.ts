import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import {Stablecoin} from "../target/types/stablecoin";
import {PythSolanaReceiver} from "@pythnetwork/pyth-solana-receiver";
import {getMint, TOKEN_2022_PROGRAM_ID, getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID} from "@solana/spl-token";

describe("stablecoin", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;
    const wallet = provider.wallet as anchor.Wallet;

    const program = anchor.workspace.Stablecoin as Program<Stablecoin>;
    const programId = program.programId;
    console.log("programId:", programId);
    const pythSolanaReceiver = new PythSolanaReceiver({connection, wallet});
    const SOL_PRICE_FEED_ID = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    const solUsdPriceFeedAccountPubkey = pythSolanaReceiver.getPriceFeedAccountAddress(0, SOL_PRICE_FEED_ID).toBase58();
    // const priceAccountPubkey = new anchor.web3.PublicKey("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");
    const timestamp = new anchor.BN(new Date().getTime()); // 使用BN处理大数
    console.log("timestamp:", timestamp.toNumber());

    const [collateralAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("collateral_account"), wallet.publicKey.toBuffer(), timestamp.toArrayLike(Buffer, 'le', 8)],
        programId
    );
    console.log("collateralAccountPDA:", collateralAccountPDA);

    const [mintAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mint_account"), timestamp.toArrayLike(Buffer, 'le', 8)],
        programId
    )
    console.log("mintAccountPDA:", mintAccountPDA);

    const [configAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("config_account"), mintAccountPDA.toBuffer(), timestamp.toArrayLike(Buffer, 'le', 8)],
        programId
    )
    console.log("configAccountPDA:", configAccountPDA);

    const [depositedAssetAccountPAD] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("deposit_asset_account"), wallet.publicKey.toBuffer(), timestamp.toArrayLike(Buffer, 'le', 8)],
        programId
    )
    console.log("depositedAssetAccountPAD:", depositedAssetAccountPAD);

    const receiveStablecoinAccount = getAssociatedTokenAddressSync(
        mintAccountPDA,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );
    console.log("receiveStablecoinAccount:", receiveStablecoinAccount);

    console.log("===========================================================================")
    it("init config......", async () => {
        // console.log("Token Program ID:", TOKEN_2022_PROGRAM_ID.toBase58());
        const sig = await program.methods
            .processInitConfig(
                timestamp,
                new anchor.BN(80),
                new anchor.BN(90),
                new anchor.BN(10),
                new anchor.BN(100),
            ).accountsStrict({
                authority: wallet.publicKey,
                mintAccount: mintAccountPDA,
                configAccount: configAccountPDA,
                systemProgram: anchor.web3.SystemProgram.programId,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
            })
            .rpc();
        console.log("===>init config sig:", sig);

        // 等待交易确认
        const latestBlockhash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
        });
        // 添加小延时确保账户更新
        await new Promise(resolve => setTimeout(resolve, 2000));

        const configAccount = await program.account.config.fetch(configAccountPDA);
        console.log("--->configAccount:", configAccount);

        const mintInfo = await getMint(
            program.provider.connection,
            mintAccountPDA,
            "confirmed",
            TOKEN_2022_PROGRAM_ID  // 指定使用 Token2022 程序
        );
        console.log("--->mintInfo:", {
            authority: mintInfo.mintAuthority?.toBase58(),
            decimals: mintInfo.decimals,
            freezeAuthority: mintInfo.freezeAuthority?.toBase58(),
            supply: mintInfo.supply.toString(),
        });

    });

    it("deposit and mint......", async () => {
        console.log("Required accounts:");
        console.log({
            depositor: wallet.publicKey.toBase58(),
            configAccount: configAccountPDA.toBase58(),
            depositedAssetAccount: depositedAssetAccountPAD.toBase58(),
            receiveStablecoinAccount: receiveStablecoinAccount.toBase58(),
            collateralAccount: collateralAccountPDA.toBase58(),
            mintAccount: mintAccountPDA.toBase58(),
            priceUpdate: solUsdPriceFeedAccountPubkey,
            systemProgram: anchor.web3.SystemProgram.programId.toBase58(),
            tokenProgram: TOKEN_2022_PROGRAM_ID.toBase58(),
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID.toBase58(),
        });

        const sig = await program.methods
            .processDepositAndMint(timestamp, new anchor.BN(8000000000))
            .accountsPartial(
                {
                    // depositor: wallet.publicKey,
                    // configAccount: configAccountPDA,
                    priceUpdate: solUsdPriceFeedAccountPubkey,
                    // mintAccount: mintAccountPDA,
                    // collateralAccount: collateralAccountPDA,
                    // depositedAssetAccount: depositedAssetAccountPAD,
                    // receiveStablecoinAccount: receiveStablecoinAccount,
                    // systemProgram: anchor.web3.SystemProgram.programId,
                    // tokenProgram: TOKEN_2022_PROGRAM_ID,
                    // associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
                }
            )
            .rpc({skipPreflight: true, commitment: "confirmed"});

        // 等待交易确认
        const latestBlockhash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
        });
        console.log("===>deposit and mint sig:", sig);

        // 获取depositedAssetAccount账户余额
        const balance = await connection.getBalance(depositedAssetAccountPAD);
        console.log(`depositedAssetAccount Balance: ${balance} lamports`);

        // 验证collateralAccount结果
        const collateralAccount = await program.account.collateral.fetch(collateralAccountPDA);
        console.log("--->collateralAccount:", collateralAccount);

        // 检查接收到的稳定币余额
        const tokenBalance = await connection.getTokenAccountBalance(receiveStablecoinAccount);
        console.log("--->stable coin token balance:", tokenBalance.value);

        //检查铸币账户在铸币后的情况
        const mintInfo = await getMint(
            connection,
            mintAccountPDA,
            "confirmed",
            TOKEN_2022_PROGRAM_ID  // 指定使用 Token2022 程序
        );
        console.log("--->after minted:", {
            authority: mintInfo.mintAuthority?.toBase58(),
            decimals: mintInfo.decimals,
            freezeAuthority: mintInfo.freezeAuthority?.toBase58(),
            supply: mintInfo.supply.toString(),
        });

    });

    it("burn stablecoin and redeem collateral......", async () => {
        const amount_to_burn = 20888713420;
        const sig = await program.methods
            .processBurnAndRedeem(timestamp, new anchor.BN(amount_to_burn))
            .accounts({priceUpdate: solUsdPriceFeedAccountPubkey})
            .rpc({skipPreflight: true, commitment: "confirmed"});
        const latestBlockhash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
        });
        const latestBlockHash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: latestBlockHash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
        });
        console.log("burn stablecoin and redeem collateral sig:", sig);

        //查看depositedAssetBalance在burn和redeem后的数据
        const depositedAssetBalance = await connection.getBalance(depositedAssetAccountPAD);
        console.log("--->deposited Asset Balance after burn and redeem:", depositedAssetBalance);

        //查看collateralAccount在burn和redeem后的数据
        const collateralAccount = await program.account.collateral.fetch(collateralAccountPDA);
        console.log("--->collateralAccount after burn and redeem:", collateralAccount);

        //查看稳定币账户在burn和redeem后的数据
        const stableCoinBalance = await connection.getTokenAccountBalance(receiveStablecoinAccount);
        console.log("--->stableCoin balance after burn and redeem::", stableCoinBalance.value);

        //检查铸币账户在燃币后的情况
        const mintInfo = await getMint(
            connection,
            mintAccountPDA,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );
        console.log("--->mint Account after burned:", {
            authority: mintInfo.mintAuthority?.toBase58(),
            decimals: mintInfo.decimals,
            freezeAuthority: mintInfo.freezeAuthority?.toBase58(),
            supply: mintInfo.supply.toString(),
        });
    });

    it("update config......", async () => {
        const sig = await program.methods.processUpdateConfig(
            timestamp,
            new anchor.BN(85),
            new anchor.BN(15),
            // new anchor.BN(95),//min_health_factor = 95时不能清算
            new anchor.BN(100),//min_health_factor = 100时不能清算
        ).rpc({skipPreflight: true, commitment: "confirmed"});
        const lastestBlockhash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: lastestBlockhash.blockhash,
            lastValidBlockHeight: lastestBlockhash.lastValidBlockHeight
        });
        console.log("--->updateConfig sig:", sig);

        const configAccount = await program.account.config.fetch(configAccountPDA);
        console.log("--->configAccount after update:", configAccount);
    })

    it("liquidate......", async () => {
        const sig = await program.methods
            .processLiquidate(timestamp, new anchor.BN(20000000000))
            .accountsPartial(
                {
                    // liquidator: wallet.publicKey,
                    collateralAccount: collateralAccountPDA,
                    priceUpdate: solUsdPriceFeedAccountPubkey,
                    // depositedAssetAccount: depositedAssetAccountPAD,
                    // liquidatorStablecoinAccount: receiveStablecoinAccount,
                    // mintAccount: mintAccountPDA,
                }
            )
            .rpc({skipPreflight: true, commitment: "confirmed"});
        const lastestBlockhash = await connection.getLatestBlockhash();
        await connection.confirmTransaction({
            signature: sig,
            blockhash: lastestBlockhash.blockhash,
            lastValidBlockHeight: lastestBlockhash.lastValidBlockHeight
        });
        console.log("--->liquidate sig:", sig);

        //查看depositedAssetBalance在liquidate后的数据
        const depositedAssetBalance = await connection.getBalance(depositedAssetAccountPAD);
        console.log("--->deposited Asset Balance after burn and redeem:", depositedAssetBalance);

        //查看collateralAccount在liquidate后的数据
        const collateralAccount = await program.account.collateral.fetch(collateralAccountPDA);
        console.log("--->collateralAccount after burn and redeem:", collateralAccount);

        //查看稳定币账户在burn和liquidate后的数据
        const stableCoinBalance = await connection.getTokenAccountBalance(receiveStablecoinAccount);
        console.log("--->stableCoin balance after burn and redeem::", stableCoinBalance.value);

        //检查铸币账户在燃币后的情况
        const mintInfo = await getMint(
            connection,
            mintAccountPDA,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );
        console.log("--->mint Account after burned:", {
            authority: mintInfo.mintAuthority?.toBase58(),
            decimals: mintInfo.decimals,
            freezeAuthority: mintInfo.freezeAuthority?.toBase58(),
            supply: mintInfo.supply.toString(),
        });
    });
});
