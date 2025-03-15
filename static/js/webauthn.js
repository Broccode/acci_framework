/**
 * WebAuthn (FIDO2) functions for browser interactions
 * These functions are called from the Rust code via wasm-bindgen
 */

// Initialize the WebAuthn module
(function() {
    // Check if browser supports WebAuthn
    if (!window.PublicKeyCredential) {
        console.warn("WebAuthn is not supported in this browser");
        return;
    }

    /**
     * Base64URL encoding/decoding functions
     */
    const base64url = {
        encode: function(buffer) {
            const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
            return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        },
        
        decode: function(base64url) {
            const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
            const binStr = atob(base64);
            const bin = new Uint8Array(binStr.length);
            for (let i = 0; i < binStr.length; i++) {
                bin[i] = binStr.charCodeAt(i);
            }
            return bin.buffer;
        }
    };

    /**
     * Convert a credential object to a format that can be sent to the server
     */
    function credentialToJSON(credential) {
        if (credential instanceof PublicKeyCredential) {
            const response = credential.response;
            const data = {
                id: credential.id,
                rawId: base64url.encode(credential.rawId),
                type: credential.type,
                response: {}
            };
            
            // Handle attestation response (registration)
            if (response instanceof AuthenticatorAttestationResponse) {
                data.response = {
                    attestationObject: base64url.encode(response.attestationObject),
                    clientDataJSON: base64url.encode(response.clientDataJSON)
                };
            }
            // Handle assertion response (authentication)
            else if (response instanceof AuthenticatorAssertionResponse) {
                data.response = {
                    authenticatorData: base64url.encode(response.authenticatorData),
                    clientDataJSON: base64url.encode(response.clientDataJSON),
                    signature: base64url.encode(response.signature),
                    userHandle: response.userHandle ? base64url.encode(response.userHandle) : null
                };
            }
            
            return data;
        }
        
        throw new Error('Not a PublicKeyCredential');
    }

    /**
     * Prepare options for registration by converting base64url-encoded values to ArrayBuffer
     */
    function prepareRegistrationOptions(options) {
        const challenge = base64url.decode(options.challenge);
        
        // Convert user.id to ArrayBuffer
        if (options.user && typeof options.user.id === 'string') {
            options.user.id = base64url.decode(options.user.id);
        }
        
        // Convert excludeCredentials to the expected format
        if (options.excludeCredentials) {
            options.excludeCredentials = options.excludeCredentials.map(credential => {
                return {
                    type: credential.type,
                    id: base64url.decode(credential.id),
                    transports: credential.transports
                };
            });
        }
        
        // Set the challenge
        options.challenge = challenge;
        
        return options;
    }

    /**
     * Prepare options for authentication by converting base64url-encoded values to ArrayBuffer
     */
    function prepareAuthenticationOptions(options) {
        const challenge = base64url.decode(options.challenge);
        
        // Convert allowCredentials to the expected format
        if (options.allowCredentials) {
            options.allowCredentials = options.allowCredentials.map(credential => {
                return {
                    type: credential.type,
                    id: base64url.decode(credential.id),
                    transports: credential.transports
                };
            });
        }
        
        // Set the challenge
        options.challenge = challenge;
        
        return options;
    }

    /**
     * Start the WebAuthn registration process
     * @param {Object} options Registration options from the server
     * @returns {Promise<Object>} Promise resolving to credential JSON
     */
    async function startRegistration(options) {
        try {
            // Prepare the options for the browser API
            const publicKeyOptions = prepareRegistrationOptions(options);
            
            // Request the browser to create a credential
            const credential = await navigator.credentials.create({
                publicKey: publicKeyOptions
            });
            
            // Convert the credential to JSON format for sending to server
            return credentialToJSON(credential);
        } catch (error) {
            console.error('WebAuthn registration error:', error);
            throw error;
        }
    }

    /**
     * Start the WebAuthn authentication process
     * @param {Object} options Authentication options from the server
     * @returns {Promise<Object>} Promise resolving to credential JSON
     */
    async function startAuthentication(options) {
        try {
            // Prepare the options for the browser API
            const publicKeyOptions = prepareAuthenticationOptions(options);
            
            // Request the browser to get a credential
            const credential = await navigator.credentials.get({
                publicKey: publicKeyOptions
            });
            
            // Convert the credential to JSON format for sending to server
            return credentialToJSON(credential);
        } catch (error) {
            console.error('WebAuthn authentication error:', error);
            throw error;
        }
    }

    // Export functions to the global scope
    window.webauthn = {
        startRegistration,
        startAuthentication
    };
})();