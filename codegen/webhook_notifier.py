#!/usr/bin/env python3

import json
import sys
import requests
from typing import Dict, Any, Optional

class WebhookNotifier:
    def __init__(self, webhook_url: str):
        self.webhook_url = webhook_url
    
    def send_notification(self, message: str, color: Optional[int] = None) -> Dict[str, Any]:
        """Send a notification to Discord webhook"""
        try:
            if color is not None:
                # Send as embed for colored messages
                payload = {
                    "embeds": [{
                        "description": message,
                        "color": color
                    }]
                }
            else:
                # Send as simple message
                payload = {
                    "content": message
                }
            
            response = requests.post(
                self.webhook_url,
                json=payload,
                headers={"Content-Type": "application/json"},
                timeout=10
            )
            
            if response.status_code == 204:
                return {
                    "success": True,
                    "message": "Notification sent successfully"
                }
            else:
                return {
                    "success": False,
                    "error": f"Failed to send notification: {response.status_code} - {response.text}"
                }
                
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def send_success(self, message: str) -> Dict[str, Any]:
        """Send success notification (green)"""
        return self.send_notification(f"✅ {message}", 0x00FF00)
    
    def send_warning(self, message: str) -> Dict[str, Any]:
        """Send warning notification (yellow)"""
        return self.send_notification(f"⚠️ {message}", 0xFFFF00)
    
    def send_error(self, message: str) -> Dict[str, Any]:
        """Send error notification (red)"""
        return self.send_notification(f"❌ {message}", 0xFF0000)
    
    def send_info(self, message: str) -> Dict[str, Any]:
        """Send info notification (blue)"""
        return self.send_notification(f"ℹ️ {message}", 0x0099FF)

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Send webhook notifications")
    parser.add_argument("--webhook-url", required=True, help="Discord webhook URL")
    parser.add_argument("--message", required=True, help="Message to send")
    parser.add_argument("--type", choices=["success", "warning", "error", "info"], default="info", help="Notification type")
    
    args = parser.parse_args()
    
    if not args.webhook_url:
        print(json.dumps({"success": False, "error": "No webhook URL provided"}))
        sys.exit(1)
    
    notifier = WebhookNotifier(args.webhook_url)
    
    if args.type == "success":
        result = notifier.send_success(args.message)
    elif args.type == "warning":
        result = notifier.send_warning(args.message)
    elif args.type == "error":
        result = notifier.send_error(args.message)
    else:
        result = notifier.send_info(args.message)
    
    print(json.dumps(result, indent=2))
    
    if not result.get("success", False):
        sys.exit(1)

if __name__ == "__main__":
    main()