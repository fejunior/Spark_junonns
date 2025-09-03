package org.jivesoftware.spark.openfire;

/**
 * Example usage of the OpenFire authentication client
 */
public class OpenFireAuthExample {
    
    public static void main(String[] args) {
        // Initialize the library
        if (!OpenFireAuthClient.initialize()) {
            System.err.println("Failed to initialize OpenFire authentication library");
            return;
        }
        
        System.out.println("OpenFire Auth Library Version: " + OpenFireAuthClient.getVersion());
        
        // Create configuration
        OpenFireAuthClient.Config config = new OpenFireAuthClient.Config("localhost", "localhost");
        config.port = 5222;
        config.use_tls = true;
        config.verify_certificates = false; // For testing with self-signed certificates
        config.resource = "SparkExample";
        
        // Create client
        OpenFireAuthClient client = null;
        try {
            client = new OpenFireAuthClient(config);
            
            // Connect and authenticate
            OpenFireAuthClient.AuthResult result = client.connect("testuser", "testpass", "localhost");
            
            if (result.success) {
                System.out.println("Successfully authenticated!");
                System.out.println("Full JID: " + result.full_jid);
                System.out.println("Session ID: " + result.session_id);
                System.out.println("Auth time: " + result.auth_time_ms + "ms");
                
                // Set presence
                if (client.setPresence(OpenFireAuthClient.PRESENCE_AVAILABLE, "Testing Rust library")) {
                    System.out.println("Presence set successfully");
                }
                
                // Get presence
                OpenFireAuthClient.Presence presence = client.getPresence();
                if (presence != null) {
                    System.out.println("Current presence: " + presence.status + " - " + presence.status_message);
                }
                
                // Send a test message
                String messageId = client.sendMessage("admin@localhost", "Hello from Rust OpenFire library!");
                if (messageId != null) {
                    System.out.println("Message sent with ID: " + messageId);
                }
                
                // Join a room
                if (client.joinRoom("testroom@conference.localhost", "TestUser")) {
                    System.out.println("Joined test room successfully");
                }
                
                // Wait a bit to see the effects
                Thread.sleep(2000);
                
            } else {
                System.err.println("Authentication failed: " + result.message);
            }
            
        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        } finally {
            // Cleanup
            if (client != null) {
                client.close();
            }
        }
    }
}