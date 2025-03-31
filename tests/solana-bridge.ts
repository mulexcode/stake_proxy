import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import {SolanaBridge} from "../target/types/solana_bridge";
import { randomBytes } from 'crypto' ;
import {secp256k1} from 'ethereum-cryptography/secp256k1'
import BN from "bn.js";
import {createMint, createMintToInstruction, getOrCreateAssociatedTokenAccount} from "@solana/spl-token";
import {expect} from "chai";
import Wallet from 'ethereumjs-wallet' ;
import {SystemProgram, Secp256k1Program, SYSVAR_INSTRUCTIONS_PUBKEY} from "@solana/web3.js";
import {keccak256} from "ethereumjs-util";
import {hexToBytes} from "ethereum-cryptography/utils";
import {Buffer} from "buffer";
import bs58 from 'bs58' ;

function uint8ToNumberArray(u8:Uint8Array) : number[] {
    const arr: number[] = [];
    for (let i = 0; i < u8.length; i++) {
        arr.push(u8[i]);
    }
    return arr;
}

function appendToMessage(destination: Uint8Array, source: Uint8Array): Uint8Array {
    const newMessage = new Uint8Array(destination.length + source.length);
    newMessage.set(destination);
    newMessage.set(source, destination.length);
    return newMessage;
}
function gen_message(nonce: bigint, magic: Uint8Array, token_name: string, token_receiver: anchor.web3.PublicKey, amount: bigint, from_chain_id: number) : Buffer {
    var message = new Uint8Array();
    message = appendToMessage(message, magic);

    const nonceBytes = new Uint8Array(8); // 假设 nonce 是 64 位整数
    new DataView(nonceBytes.buffer).setBigUint64(0, nonce, true); // true 表示小端序
    message = appendToMessage(message, nonceBytes);
    message = appendToMessage(message, new TextEncoder().encode(token_name));

// 4. 添加 token_receiver 的公钥字节
    message = appendToMessage(message, token_receiver.toBytes());

// 5. 添加 amount（小端序）
    const amountBytes = new Uint8Array(8); // 假设 amount 是 64 位整数
    new DataView(amountBytes.buffer).setBigUint64(0, amount, true);
    message = appendToMessage(message, amountBytes);

// 6. 添加 from_chain_id（小端序）
    const chainIdBytes = new Uint8Array(8); // 假设 chain_id 是 32 位整数
    new DataView(chainIdBytes.buffer).setUint32(0, from_chain_id, true);
    message = appendToMessage(message, chainIdBytes);

    const buff = new Buffer(message)
    console.log("buff", buff.toString("hex"))
    return buff
}
describe("solana-bridge", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.SolanaBridge as Program<SolanaBridge>;
    const [nativeVault, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("native_token_vault")], program.programId);
    console.log("nativeVault: ", nativeVault.toString())
    const payer = anchor.Wallet.local().payer;
    const stakeMintKeyPair = anchor.web3.Keypair.generate();
    const [systemConfig, ]= anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("system_config")], program.programId);
    const ethWallet = Wallet.fromPrivateKey(Buffer.from(randomBytes(32)));
    const ethAddress = ethWallet.getAddress();
    const [chainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(1).toArrayLike(Buffer, 'le', 8)], program.programId);
    const [solChainConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("chain_config"), new BN(101).toArrayLike(Buffer, 'le', 8)], program.programId);
    const [tokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("test")], program.programId);
    const [solTokenConfig, ] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("token_config"), Buffer.from("sol")], program.programId);
    const anotherPayer = anchor.web3.Keypair.generate();
    console.log("??", (new BN(1).toArrayLike(Buffer, 'le', 8)).toString("hex"), chainConfig.toString())
    console.log("native", nativeVault.toString());

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

    async function requestToken(staker :anchor.web3.PublicKey, amount: number, authority: anchor.web3.Keypair) {
        let ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            payer,
            stakeMintKeyPair.publicKey, // mint
            staker, // owner
        );
        let tx = new anchor.web3.Transaction().add(
            createMintToInstruction(stakeMintKeyPair.publicKey, ata.address, authority.publicKey, amount, [authority])
        );
        console.log(
            `requestStakeToken txhash: ${await anchor.getProvider().sendAndConfirm(tx, [payer])}`,
        );
        return {ata}
    }


    before(async () => {
        await program.methods.initialize(new BN(10086), payer.publicKey, uint8ToNumberArray(ethAddress)).accountsPartial({
            nativeTokenVault: nativeVault,
            payer: payer.publicKey,
        }).signers([payer]).rpc();
        await createMint(anchor.getProvider().connection, payer, nativeVault, payer.publicKey,  9, stakeMintKeyPair);
        await program.methods.enableChain(new BN(1)).accountsPartial({
            payer: payer.publicKey,
        }).signers([payer]).rpc();

        await program.methods.enableToken("test").accountsPartial({
            payer: payer.publicKey,
            tokenMint: stakeMintKeyPair.publicKey,
        }).signers([payer]).rpc();

        await program.methods.enableChain(new BN(101)).accountsPartial({
            payer: payer.publicKey,
        }).signers([payer]).rpc();

        await program.methods.enableToken("sol").accountsPartial({
            payer: payer.publicKey,
            tokenMint: stakeMintKeyPair.publicKey,
        }).signers([payer]).rpc();

        await requestAirdrop(anotherPayer.publicKey, 100000000000);
    })


    it("initialize", async () => {
        const systemConfigData = await program.account.systemConfig.fetch(systemConfig);
        expect(systemConfigData.chainId.toString()).eq("10086");
        expect(systemConfigData.manager.toString()).eq(payer.publicKey.toString());
        expect(systemConfigData.secp256K1Manager.toString()).eq(uint8ToNumberArray(ethAddress).toString())
    });

    it("token", async () => {
        let ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            payer,
            stakeMintKeyPair.publicKey, // mint
            anotherPayer.publicKey, // owner
        );
        const message = gen_message(BigInt(1), new Uint8Array([1,2,3,4,5,6,7,8]), "test", ata.address, BigInt(1000000), 1);
        const signature = secp256k1.sign(keccak256(message).toString("hex"), ethWallet.getPrivateKey());

        console.log(ethWallet.getPrivateKey().toString("hex"), message.toString("hex"), signature.toCompactHex(), signature.recovery)

        console.log(ethAddress.toString("hex"))

        const ed25519Instruction = Secp256k1Program.createInstructionWithEthAddress({
            ethAddress: ethWallet.getAddress(),
            message: message,
            signature: hexToBytes(signature.toCompactHex()),
            recoveryId: signature.recovery,
            instructionIndex: 0
        });

        const payoutIns = await program.methods.payout({
            tokenName: "test",
            amount: new BN(1000000),
            fromChainId: new BN(1),
            magic: [1,2,3,4,5,6,7,8],
            nonce: new BN(1),
            signature: uint8ToNumberArray(hexToBytes(signature.toCompactHex())),
            recoveryId: signature.recovery,
        }).accountsPartial({
            tokenReceiver: ata.address,
            chainConfig: chainConfig,
            tokenMint: stakeMintKeyPair.publicKey,
            tokenConfig: tokenConfig,
            instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        }).instruction();

        let tx = new anchor.web3.Transaction().add(
            ed25519Instruction,
            payoutIns
        );
        console.log(
            `initializeAccount txhash: ${await anchor.getProvider().sendAndConfirm(tx, [])}`,
        );

        await program.methods.cashOut({
                    tokenName: "test",
                    target: "testAddress",
                    amount: new BN(100),
                    targetChainId: new BN(1),
                }).accountsPartial({
                    payer: anotherPayer.publicKey,
                    chainConfig: chainConfig,
                    tokenMint: stakeMintKeyPair.publicKey,
                    tokenConfig: tokenConfig,
                    tokenPayer: ata.address,
                }).signers([anotherPayer]).rpc();
        const logs = await anchor.getProvider().connection.getParsedTransaction(bs58.encode(tx.signature) ,{ commitment : "confirmed" } ) ;
        console.log(bs58.encode(tx.signature) , logs);
    });

    it("sol", async () => {
        await program.methods.cashOutSol(new BN(101), anotherPayer.publicKey, new BN(10000)).accountsPartial({
            payer: anotherPayer.publicKey,
            chainConfig: solChainConfig,
        }).signers([anotherPayer]).rpc();

        const message = gen_message(BigInt(1), new Uint8Array([1,2,3,4,5,6,7,8]), "sol", payer.publicKey, BigInt(10000), 101);
        const signature = secp256k1.sign(keccak256(message).toString("hex"), ethWallet.getPrivateKey());

        console.log(ethWallet.getPrivateKey().toString("hex"), message.toString("hex"), signature.toCompactHex(), signature.recovery)

        console.log(ethAddress.toString("hex"))

        const ed25519Instruction = Secp256k1Program.createInstructionWithEthAddress({
            ethAddress: ethWallet.getAddress(),
            message: message,
            signature: hexToBytes(signature.toCompactHex()),
            recoveryId: signature.recovery,
            instructionIndex: 0
        });

        const payoutIns = await program.methods.payoutSol({
            amount: new BN(10000),
            fromChainId: new BN(101),
            magic: [1,2,3,4,5,6,7,8],
            nonce: new BN(1),
            signature: uint8ToNumberArray(hexToBytes(signature.toCompactHex())),
            recoveryId: signature.recovery,
        }).accountsPartial({
            receiver: payer.publicKey,
            chainConfig: solChainConfig,
            tokenConfig: solTokenConfig,
            instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        }).instruction();

        let tx = new anchor.web3.Transaction().add(
            ed25519Instruction,
            payoutIns
        );
        console.log(
            `initializeAccount txhash: ${await anchor.getProvider().sendAndConfirm(tx, [])}`,
        );
        const logs = await anchor.getProvider().connection.getParsedTransaction(bs58.encode(tx.signature) ,{ commitment : "confirmed" } ) ;
        console.log(bs58.encode(tx.signature) , logs);
    });
})