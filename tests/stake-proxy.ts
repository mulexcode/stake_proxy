import * as anchor from "@coral-xyz/anchor";
import {Program, web3} from "@coral-xyz/anchor";
import {StakeProxy} from "../target/types/stake_proxy";
import {PublicKey, SystemProgram} from "@solana/web3.js" ;
import {
    createInitializeMint2Instruction, createMintToInstruction,
    createSyncNativeInstruction,
    getAssociatedTokenAddress, getAssociatedTokenAddressSync,
    getOrCreateAssociatedTokenAccount, createMint,
    NATIVE_MINT
} from "@solana/spl-token";
import BN from "bn.js";
import {createAssociatedTokenAccountInstruction} from "@solana/spl-token/src/instructions/associatedTokenAccount";

describe("stake-proxy", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')

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
        console.log("init")
        const {ata} = await requestWsol(payer)
        console.log("init2")
        return {nativeVault, ata}
    }

    async function requestWsol(payer: anchor.web3.Keypair) {
        const mint = anchor.web3.Keypair.fromSecretKey(new Uint8Array([213,50,144,69,65,59,187,93,206,199,77,226,167,39,149,36,254,45,245,78,48,69,138,238,83,154,149,142,144,116,208,229,28,140,211,193,128,241,114,45,179,80,71,32,158,243,51,166,90,93,254,59,46,156,238,126,126,142,21,9,97,38,161,26]))

        await createMint(anchor.getProvider().connection, payer, payer.publicKey, payer.publicKey,  9, mint)

        // remember to create ATA first
        let ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            payer,
            mint.publicKey, // mint
            payer.publicKey, // owner
        );
        

        let tx = new anchor.web3.Transaction().add(
            createMintToInstruction(mint.publicKey, ata.address, payer.publicKey, 100000000 * anchor.web3.LAMPORTS_PER_SOL, [payer])
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

        const WSOL = new anchor.web3.PublicKey("2vSxuEFrcRCrj95jGnQ5kfSPMN2JyqXhfA22iaiivR7f")
        const tokenVault = getAssociatedTokenAddressSync(WSOL, nativeVault, true);

        const createAccountInstruction = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: sys_stake_state.publicKey,
            lamports: anchor.web3.LAMPORTS_PER_SOL,
            space: 200,
            programId: new PublicKey("Stake11111111111111111111111111111111111111")
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
