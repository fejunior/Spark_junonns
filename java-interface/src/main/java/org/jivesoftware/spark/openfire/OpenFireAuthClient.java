package org.jivesoftware.spark.openfire;

import com.google.gson.Gson;
import com.google.gson.JsonSyntaxException;
import java.util.logging.Logger;
import java.util.logging.Level;

/**
 * High-level Java wrapper for the Rust OpenFire authentication library.
 * This class provides a convenient Java API for OpenFire authentication and communication.
 */
public class OpenFireAuthClient {
    
    private static final Logger LOGGER = Logger.getLogger(OpenFireAuthClient.class.getName());
    private static final Gson gson = new Gson();
    private static boolean isInitialized = false;
    
    private long clientId = -1;
    private boolean connected = false;
    
    // Configuration class for JSON serialization
    public static class Config {
        public String server;
        public int port = 5222;
        public String domain;
        public boolean use_tls = true;
        public boolean verify_certificates = true;
        public long connection_timeout = 30;
        public long auth_timeout = 10;
        public String resource = "SparkJava";
        public int priority = 1;
        
        public Config(String server, String domain) {
            this.server = server;
            this.domain = domain;
        }
    }
    
    // Authentication result class
    public static class AuthResult {
        public boolean success;
        public String message;
        public String full_jid;
        public String session_id;
        public long auth_time_ms;
    }
    
    // Presence information class
    public static class Presence {
        public String jid;
        public String status; // Available, Away, ExtendedAway, DoNotDisturb, Unavailable, Invisible
        public String status_message;
        public int priority;
        public long timestamp;
    }
    
    // Presence status constants
    public static final int PRESENCE_AVAILABLE = 0;
    public static final int PRESENCE_AWAY = 1;
    public static final int PRESENCE_EXTENDED_AWAY = 2;
    public static final int PRESENCE_DO_NOT_DISTURB = 3;
    public static final int PRESENCE_UNAVAILABLE = 4;
    public static final int PRESENCE_INVISIBLE = 5;
    
    /**
     * Initialize the OpenFire authentication library (call once per application)
     * @return true if initialization was successful
     */
    public static synchronized boolean initialize() {
        if (!isInitialized) {
            try {
                isInitialized = OpenFireAuthNative.initialize();
                if (isInitialized) {
                    LOGGER.info("OpenFire authentication library initialized successfully");
                } else {
                    LOGGER.severe("Failed to initialize OpenFire authentication library");
                }
            } catch (Exception e) {
                LOGGER.log(Level.SEVERE, "Exception during OpenFire library initialization", e);
                isInitialized = false;
            }
        }
        return isInitialized;
    }
    
    /**
     * Create a new OpenFire client
     * @param config Configuration for the client
     */
    public OpenFireAuthClient(Config config) {
        if (!isInitialized) {
            throw new IllegalStateException("OpenFire library not initialized. Call initialize() first.");
        }
        
        try {
            String configJson = gson.toJson(config);
            this.clientId = OpenFireAuthNative.createClient(configJson);
            if (this.clientId < 0) {
                throw new RuntimeException("Failed to create OpenFire client");
            }
            LOGGER.info("Created OpenFire client with ID: " + this.clientId);
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Failed to create OpenFire client", e);
            throw new RuntimeException("Failed to create OpenFire client", e);
        }
    }
    
    /**
     * Connect to OpenFire server and authenticate
     * @param username Username for authentication
     * @param password Password for authentication
     * @param domain Domain for authentication (optional, can be null or empty)
     * @return AuthResult containing the result of authentication
     */
    public AuthResult connect(String username, String password, String domain) {
        if (clientId < 0) {
            throw new IllegalStateException("Client not properly initialized");
        }
        
        try {
            String domainStr = (domain != null) ? domain : "";
            String resultJson = OpenFireAuthNative.connect(clientId, username, password, domainStr);
            
            if (resultJson == null) {
                AuthResult result = new AuthResult();
                result.success = false;
                result.message = "Connection failed - no response from native library";
                return result;
            }
            
            AuthResult result = gson.fromJson(resultJson, AuthResult.class);
            this.connected = result.success;
            
            if (result.success) {
                LOGGER.info("Successfully connected to OpenFire server: " + result.full_jid);
            } else {
                LOGGER.warning("Failed to connect to OpenFire server: " + result.message);
            }
            
            return result;
        } catch (JsonSyntaxException e) {
            LOGGER.log(Level.SEVERE, "Failed to parse authentication result JSON", e);
            AuthResult result = new AuthResult();
            result.success = false;
            result.message = "JSON parsing error: " + e.getMessage();
            return result;
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Exception during connection", e);
            AuthResult result = new AuthResult();
            result.success = false;
            result.message = "Connection exception: " + e.getMessage();
            return result;
        }
    }
    
    /**
     * Disconnect from OpenFire server
     * @return true if disconnected successfully
     */
    public boolean disconnect() {
        if (clientId < 0) {
            return false;
        }
        
        try {
            boolean result = OpenFireAuthNative.disconnect(clientId);
            if (result) {
                this.connected = false;
                LOGGER.info("Disconnected from OpenFire server");
            } else {
                LOGGER.warning("Failed to disconnect from OpenFire server");
            }
            return result;
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Exception during disconnection", e);
            return false;
        }
    }
    
    /**
     * Check if connected to the server
     * @return true if connected
     */
    public boolean isConnected() {
        if (clientId < 0) {
            return false;
        }
        
        try {
            boolean nativeConnected = OpenFireAuthNative.isConnected(clientId);
            this.connected = nativeConnected;
            return nativeConnected;
        } catch (Exception e) {
            LOGGER.log(Level.WARNING, "Exception checking connection status", e);
            this.connected = false;
            return false;
        }
    }
    
    /**
     * Send a chat message
     * @param to Recipient JID
     * @param body Message body
     * @return Message ID on success, null on failure
     */
    public String sendMessage(String to, String body) {
        if (!isConnected()) {
            LOGGER.warning("Cannot send message - not connected to server");
            return null;
        }
        
        try {
            String messageId = OpenFireAuthNative.sendMessage(clientId, to, body);
            if (messageId != null) {
                LOGGER.info("Sent message to " + to + ": " + messageId);
            } else {
                LOGGER.warning("Failed to send message to " + to);
            }
            return messageId;
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Exception sending message", e);
            return null;
        }
    }
    
    /**
     * Set presence status
     * @param status Presence status (use PRESENCE_* constants)
     * @param message Status message (optional)
     * @return true if presence was set successfully
     */
    public boolean setPresence(int status, String message) {
        if (!isConnected()) {
            LOGGER.warning("Cannot set presence - not connected to server");
            return false;
        }
        
        try {
            String statusMessage = (message != null) ? message : "";
            boolean result = OpenFireAuthNative.setPresence(clientId, status, statusMessage);
            if (result) {
                LOGGER.info("Presence updated: status=" + status + ", message=" + statusMessage);
            } else {
                LOGGER.warning("Failed to update presence");
            }
            return result;
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Exception setting presence", e);
            return false;
        }
    }
    
    /**
     * Get current presence information
     * @return Presence object, null if not available
     */
    public Presence getPresence() {
        if (clientId < 0) {
            return null;
        }
        
        try {
            String presenceJson = OpenFireAuthNative.getPresence(clientId);
            if (presenceJson == null) {
                return null;
            }
            
            return gson.fromJson(presenceJson, Presence.class);
        } catch (JsonSyntaxException e) {
            LOGGER.log(Level.WARNING, "Failed to parse presence JSON", e);
            return null;
        } catch (Exception e) {
            LOGGER.log(Level.WARNING, "Exception getting presence", e);
            return null;
        }
    }
    
    /**
     * Join a chat room
     * @param roomJid Room JID
     * @param nickname Nickname to use in the room
     * @return true if joined successfully
     */
    public boolean joinRoom(String roomJid, String nickname) {
        if (!isConnected()) {
            LOGGER.warning("Cannot join room - not connected to server");
            return false;
        }
        
        try {
            boolean result = OpenFireAuthNative.joinRoom(clientId, roomJid, nickname);
            if (result) {
                LOGGER.info("Joined room: " + roomJid + " as " + nickname);
            } else {
                LOGGER.warning("Failed to join room: " + roomJid);
            }
            return result;
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, "Exception joining room", e);
            return false;
        }
    }
    
    /**
     * Get the library version
     * @return Version string
     */
    public static String getVersion() {
        try {
            return OpenFireAuthNative.getVersion();
        } catch (Exception e) {
            LOGGER.log(Level.WARNING, "Exception getting version", e);
            return "unknown";
        }
    }
    
    /**
     * Cleanup resources
     */
    public void close() {
        if (clientId >= 0) {
            try {
                if (connected) {
                    disconnect();
                }
                OpenFireAuthNative.destroyClient(clientId);
                LOGGER.info("Destroyed OpenFire client: " + clientId);
            } catch (Exception e) {
                LOGGER.log(Level.WARNING, "Exception during cleanup", e);
            } finally {
                clientId = -1;
                connected = false;
            }
        }
    }
    
    @Override
    protected void finalize() throws Throwable {
        try {
            close();
        } finally {
            super.finalize();
        }
    }
}