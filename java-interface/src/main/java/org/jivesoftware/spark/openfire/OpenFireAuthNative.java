package org.jivesoftware.spark.openfire;

/**
 * Native interface to the Rust OpenFire authentication library.
 * This class provides JNI bindings to interact with the Rust-based
 * authentication and communication module.
 */
public class OpenFireAuthNative {
    
    // Load the native library
    static {
        try {
            System.loadLibrary("openfire_auth");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load openfire_auth native library: " + e.getMessage());
            throw e;
        }
    }
    
    /**
     * Initialize the OpenFire authentication library
     * @return true if initialization was successful, false otherwise
     */
    public static native boolean initialize();
    
    /**
     * Create a new OpenFire client instance
     * @param configJson Configuration in JSON format
     * @return Client ID (positive number) on success, -1 on failure
     */
    public static native long createClient(String configJson);
    
    /**
     * Destroy an OpenFire client instance
     * @param clientId Client ID returned from createClient
     * @return true if client was destroyed successfully, false otherwise
     */
    public static native boolean destroyClient(long clientId);
    
    /**
     * Connect to OpenFire server and authenticate
     * @param clientId Client ID
     * @param username Username for authentication
     * @param password Password for authentication
     * @param domain Domain for authentication (can be empty)
     * @return JSON string containing authentication result
     */
    public static native String connect(long clientId, String username, String password, String domain);
    
    /**
     * Disconnect from OpenFire server
     * @param clientId Client ID
     * @return true if disconnected successfully, false otherwise
     */
    public static native boolean disconnect(long clientId);
    
    /**
     * Check if client is connected to the server
     * @param clientId Client ID
     * @return true if connected, false otherwise
     */
    public static native boolean isConnected(long clientId);
    
    /**
     * Send a chat message
     * @param clientId Client ID
     * @param to Recipient JID
     * @param body Message body
     * @return Message ID on success, null on failure
     */
    public static native String sendMessage(long clientId, String to, String body);
    
    /**
     * Set presence status
     * @param clientId Client ID
     * @param status Presence status (0=Available, 1=Away, 2=ExtendedAway, 3=DoNotDisturb, 4=Unavailable, 5=Invisible)
     * @param message Status message (can be null or empty)
     * @return true if presence was set successfully, false otherwise
     */
    public static native boolean setPresence(long clientId, int status, String message);
    
    /**
     * Get current presence as JSON
     * @param clientId Client ID
     * @return JSON string containing presence information, null if not available
     */
    public static native String getPresence(long clientId);
    
    /**
     * Join a chat room
     * @param clientId Client ID
     * @param roomJid Room JID
     * @param nickname Nickname to use in the room
     * @return true if joined successfully, false otherwise
     */
    public static native boolean joinRoom(long clientId, String roomJid, String nickname);
    
    /**
     * Get library version
     * @return Version string
     */
    public static native String getVersion();
}