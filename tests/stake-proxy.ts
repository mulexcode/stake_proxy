import * as anchor from "@coral-xyz/anchor";
import {Program, web3} from "@coral-xyz/anchor";
import {StakeProxy} from "../target/types/stake_proxy";
import {PublicKey, SystemProgram, VoteInit, VoteProgram} from "@solana/web3.js" ;
import {
    createInitializeMint2Instruction, createMintToInstruction,
    createSyncNativeInstruction,
    getAssociatedTokenAddress, getAssociatedTokenAddressSync,
    getOrCreateAssociatedTokenAccount, createMint,
    NATIVE_MINT, getAccount, getMinimumBalanceForRentExemptAccount
} from "@solana/spl-token";
import BN from "bn.js";
import {createAssociatedTokenAccountInstruction} from "@solana/spl-token/src/instructions/associatedTokenAccount";
import {expect} from "chai";

describe("stake-proxy", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.StakeProxy as Program<StakeProxy>;

    const [nativeVault, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("native_vault")], program.programId)
    const payer = anchor.Wallet.local().payer;
    const stakeMintKeyPair = anchor.web3.Keypair.fromSecretKey(new Uint8Array([213,50,144,69,65,59,187,93,206,199,77,226,167,39,149,36,254,45,245,78,48,69,138,238,83,154,149,142,144,116,208,229,28,140,211,193,128,241,114,45,179,80,71,32,158,243,51,166,90,93,254,59,46,156,238,126,126,142,21,9,97,38,161,26]))
    const stakeMint = stakeMintKeyPair.publicKey;
    const tokenVault = getAssociatedTokenAddressSync(stakeMint, nativeVault, true);
    const voter = anchor.web3.Keypair.generate();

    async function requestAirdrop(to: anchor.web3.PublicKey,  amount: number) {
        const ix = SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: to,
            lamports: amount,
        })
        let tx = new anchor.web3.Transaction().add(
            ix,
        );
        console.log(
            `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer])}`,
        );
    }

    before(async () => {
        await program.methods.initialize().accountsPartial({
            nativeVault: nativeVault,
            payer: payer.publicKey,
        }).signers([payer]).rpc()
        await requestAirdrop(nativeVault, 10000 * anchor.web3.LAMPORTS_PER_SOL);
        await createMint(anchor.getProvider().connection, payer, payer.publicKey, payer.publicKey,  9, stakeMintKeyPair)
        await createVoter();
    })

    async function requestStakeToken(staker :anchor.web3.PublicKey, amount: number, authority: anchor.web3.Keypair) {
        let ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            payer,
            stakeMint, // mint
            staker, // owner
        );
        let tx = new anchor.web3.Transaction().add(
            createMintToInstruction(stakeMint, ata.address, authority.publicKey, amount, [authority])
        );
        console.log(
            `requestStakeToken txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer])}`,
        );
        return {ata}
    }

    async function createVoter() {
        const nodeKey = anchor.web3.Keypair.generate();
        const auth1 = anchor.web3.Keypair.generate();
        const auth2 = anchor.web3.Keypair.generate();
        const tx = VoteProgram.createAccount({
            fromPubkey: payer.publicKey,
            votePubkey: voter.publicKey,
            voteInit: new VoteInit(nodeKey.publicKey, auth1.publicKey, auth2.publicKey, 1),
            lamports: anchor.web3.LAMPORTS_PER_SOL,
        })
        console.log(
            `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer, nodeKey, voter])}`,
        );
    }

    it("initialize account", async () => {
        const newStaker = anchor.web3.Keypair.generate();
        await requestAirdrop(newStaker.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL);
        const sys_stake_state = anchor.web3.Keypair.generate()
        const [stakeInfo, stakeInfoBump]  = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("stake_info"), sys_stake_state.publicKey.toBuffer()], program.programId)
        const initBalance = anchor.web3.LAMPORTS_PER_SOL;

        const createAccountInstruction = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: sys_stake_state.publicKey,
            lamports: initBalance,
            space: 200,
            programId: new PublicKey("Stake11111111111111111111111111111111111111")
        });
        const {ata} = await requestStakeToken(newStaker.publicKey, 100*anchor.web3.LAMPORTS_PER_SOL, payer)
        const initializeAccountInstruction = await program.methods.initializeAccount({
            amount: new BN(2*anchor.web3.LAMPORTS_PER_SOL),
            staker: newStaker.publicKey,
            withdrawer: newStaker.publicKey,
        }).accountsPartial({
            sysStakeState: sys_stake_state.publicKey,
            stakeInfo: stakeInfo,
            tokenVault: tokenVault,
            nativeVault: nativeVault,
            tokenMint: stakeMint,
            tokenPayer: ata.address,
            payer: newStaker.publicKey,
        }).instruction()
        let tx = new anchor.web3.Transaction().add(
            createAccountInstruction,
            initializeAccountInstruction
        );
        console.log(
            `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [newStaker, sys_stake_state])}`,
        );
        const stakeInfoData = await program.account.stakeInfo.fetch(stakeInfo);
        const tokenVaultBalance = (await getAccount(anchor.getProvider().connection, tokenVault)).amount
        expect(stakeInfoData.amount.toString()).eq(tokenVaultBalance.toString());
        expect(stakeInfoData.stakerPubkey.toString()).to.be.equals(newStaker.publicKey.toString());
        expect(stakeInfoData.withdrawerPubkey.toString()).to.be.equals(newStaker.publicKey.toString());

        const stakeStateBalance = await anchor.getProvider().connection.getBalance(sys_stake_state.publicKey);
        const minAmount = await anchor.getProvider().connection.getMinimumBalanceForRentExemption(200)
        expect((stakeStateBalance-minAmount).toString()).to.be.equals(stakeInfoData.amount.toString());

        await program.methods.delegateStake({amount: new BN(3*anchor.web3.LAMPORTS_PER_SOL)}).accountsPartial({
            authority: newStaker.publicKey,
            nativeVault: nativeVault,
            stakeInfo: stakeInfo,
            sysStakeState: sys_stake_state.publicKey,
            tokenPayer: ata.address,
            tokenVault: tokenVault,
            vote: voter.publicKey,
            stakeConfig: new anchor.web3.PublicKey("StakeConfig11111111111111111111111111111111")
        }).signers([newStaker]).rpc()

        const stakeInfoDataAfterDelegate = await program.account.stakeInfo.fetch(stakeInfo);
        const tokenVaultBalanceAfterDelegate = (await getAccount(anchor.getProvider().connection, tokenVault)).amount
        expect(stakeInfoDataAfterDelegate.amount.toString()).eq((new BN(3*anchor.web3.LAMPORTS_PER_SOL)).toString());
        expect(stakeInfoDataAfterDelegate.amount.toString()).eq(tokenVaultBalanceAfterDelegate.toString());

        const stakeStateBalanceAfter = await anchor.getProvider().connection.getBalance(sys_stake_state.publicKey);
        expect((stakeStateBalanceAfter-minAmount).toString()).to.be.equals(stakeInfoDataAfterDelegate.amount.toString());

        try {
            await program.methods.withdraw({amount: new BN(3*anchor.web3.LAMPORTS_PER_SOL)}).accountsPartial({
                authority: newStaker.publicKey,
                stakeInfo: stakeInfo,
                sysStakeState: sys_stake_state.publicKey,
                tokenVault: tokenVault,
                nativeVault: nativeVault,
                tokenReceiver: ata.address,
            }).signers([newStaker]).rpc()
        } catch (e) {
            console.log(e)
            expect(e.transactionMessage).to.be.eq("Transaction simulation failed: Error processing Instruction 0: insufficient funds for instruction");
        }

        await program.methods.deactivate().accountsPartial({
            stakeInfo: stakeInfo,
            sysStakeState: sys_stake_state.publicKey,
            authority: newStaker.publicKey,
        }).signers([newStaker]).rpc()

        await program.methods.withdraw({amount: new BN(3*anchor.web3.LAMPORTS_PER_SOL)}).accountsPartial({
            authority: newStaker.publicKey,
            stakeInfo: stakeInfo,
            sysStakeState: sys_stake_state.publicKey,
            tokenVault: tokenVault,
            nativeVault: nativeVault,
            tokenReceiver: ata.address,
        }).signers([newStaker]).rpc()

        const stakeInfoDataAfterWithdraw = await program.account.stakeInfo.fetch(stakeInfo);
        const tokenVaultBalanceAfterWithdraw = (await getAccount(anchor.getProvider().connection, tokenVault)).amount
        expect(stakeInfoDataAfterWithdraw.amount.toString()).eq("0");
        expect(stakeInfoDataAfterWithdraw.amount.toString()).eq(tokenVaultBalanceAfterWithdraw.toString());

        const stakeStateBalanceWithdraw = await anchor.getProvider().connection.getBalance(sys_stake_state.publicKey);
        expect(stakeStateBalanceWithdraw.toString()).to.be.equals(minAmount.toString());
    });
});
