// src/lib/types.ts

// Basic types remain string for serialization
// import type { DateTime } from 'chrono';
// import type { Uuid } from 'uuid';

/**
 * Represents core descriptive information about a cryptographic key.
 * Matches Rust struct `KeyInfo`. Uses camelCase.
 */
export type KeyInfo = {
    /** Unique identifier for the key pair. */
    keyId: string;       // <-- Changed to camelCase
    /** User-defined name for the key. */
    name: string;        // <-- Already camelCase
    /** The signature algorithm associated with this key. */
    algorithm: string;   // <-- Already camelCase
    /** Timestamp (UTC ISO 8601 string) when the key was generated or imported. */
    createdAt: string;   // <-- Changed to camelCase
};

/**
 * Represents detailed information about a cryptographic key, including the public key PEM.
 * Matches Rust struct `KeyDetails`. Uses camelCase.
 */
export type KeyDetails = {
    /** Core information about the key. */
    info: KeyInfo;       // <-- Already camelCase
    /** The public key encoded in PEM (SPKI) format. */
    publicKeyPem: string; // <-- Changed to camelCase
};

/**
 * Enumerates the supported signature algorithms (string literals matching Rust Display).
 */
export type SignatureAlgorithm =
    | "RSA-PKCS1-SHA256"
    | "ECDSA-P256-SHA256"
    | "Ed25519";

/**
 * Specifies the desired format for the output signature.
 * Matches Rust enum `SignatureFormat`. Uses camelCase if specified in Rust.
 */
export type SignatureFormat = 'detached'; // <-- Value likely remains lowercase

/**
 * Options provided when invoking the signing command.
 * Matches Rust struct `SigningOptions`. Uses camelCase.
 */
export type SigningOptions = {
    /** The desired output format for the signature. */
    format: SignatureFormat; // <-- Already camelCase
};

/**
 * Represents the outcome of a signature verification attempt.
 * Matches Rust struct `VerificationResult`. Uses camelCase.
 */
export type VerificationResult = {
    /** Indicates whether the cryptographic verification was successful. */
    isValid: boolean;      // <-- Changed to camelCase
    /** Provides details if verification failed or if a processing error occurred. */
    errorMessage?: string | null; // <-- Changed to camelCase, made optional '?' due to skip_serializing_if
};

/**
 * Represents the list of algorithms supported by the frontend for generation.
 */
export const supportedAlgorithmsForGeneration: { value: SignatureAlgorithm; label: string }[] = [
    { value: "RSA-PKCS1-SHA256", label: "RSA 2048 (PKCS#1 SHA-256)" },
    { value: "ECDSA-P256-SHA256", label: "ECDSA P-256 (SHA-256)" },
    { value: "Ed25519", label: "Ed25519" },
];
