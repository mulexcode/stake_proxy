import * as anchor from "@coral-xyz/anchor";
import {Program, web3} from "@coral-xyz/anchor";
import {StakeProxy} from "../target/types/stake_proxy";
import {SystemProgram} from "@solana/web3.js" ;
import {
    createSyncNativeInstruction,
    getAssociatedTokenAddress, getAssociatedTokenAddressSync,
    getOrCreateAssociatedTokenAccount,
    NATIVE_MINT
} from "@solana/spl-token";
import BN from "bn.js";

describe("stake-proxy", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.StakeProxy as Program<StakeProxy>;

    async function initialize() {
        const [nativeVault, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("native_vault")], program.programId)
        const payer = anchor.Wallet.local().payer;
        const s0 = await anchor.getProvider().connection.requestAirdrop(payer.publicKey, 10000000 * anchor.web3.LAMPORTS_PER_SOL);

       await program.methods.initialize().accountsPartial({
            nativeVault: nativeVault,
            payer: payer.publicKey,
        }).signers([payer]).rpc()


        const s1 = await anchor.getProvider().connection.requestAirdrop(nativeVault, 1000000 * anchor.web3.LAMPORTS_PER_SOL);
        await anchor.getProvider().connection.confirmTransaction(s1);
        const {ata} = await requestWsol(payer)
        return {nativeVault, ata}
    }

    async function requestWsol(payer: anchor.web3.Keypair) {

        // remember to create ATA first
        let ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            payer,
            NATIVE_MINT, // mint
            payer.publicKey, // owner
        );

        let amount = 100 * 1e9; /* Wrapped SOL's decimals is 9 */

        let tx = new anchor.web3.Transaction().add(
            // transfer SOL
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: ata.address,
                lamports: amount,
            }),
            // sync wrapped SOL balance
            createSyncNativeInstruction(ata.address),
        );
        console.log(
            `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer])}`,
        );
        return {ata}
    }

    it("Is initialized!", async () => {
        const payer = anchor.Wallet.local().payer;
        // Add your test here.
        const {nativeVault, ata} = await initialize()
        const sys_stake_state = anchor.web3.Keypair.generate()
        const [stakeInfo, stakeInfoBump]  = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("stake_info"), sys_stake_state.publicKey.toBuffer()], program.programId)

        const WSOL = new anchor.web3.PublicKey("So11111111111111111111111111111111111111112")
        const tokenVault = getAssociatedTokenAddressSync(WSOL, nativeVault, true);

        const createAccountInstruction = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: sys_stake_state.publicKey,
            lamports: anchor.web3.LAMPORTS_PER_SOL,
            space: 0,
            programId: program.programId
        });

        const initializeAccountInstruction = await program.methods.initializeAccount({
            amount: new BN(2*anchor.web3.LAMPORTS_PER_SOL),
            staker: payer.publicKey,
            withdrawer: payer.publicKey,
        }).accountsPartial({
            sysStakeState: sys_stake_state.publicKey,
            stakeInfo: stakeInfo,
            tokenVault: tokenVault,
            nativeVault: nativeVault,
            tokenMint: WSOL,
            tokenPayer: ata.address,
            payer: payer.publicKey,
        }).instruction()
        let tx = new anchor.web3.Transaction().add(
            createAccountInstruction,
            initializeAccountInstruction
        );
        console.log(
            `txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer, sys_stake_state])}`,
        );
    });
});
