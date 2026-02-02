import argparse
import json
import requests
import sys
import os
import time

def run_test(name, func):
    print(f"Testing {name}...", end="", flush=True)
    try:
        func()
        print(" ✅ PASS")
        return True
    except Exception as e:
        print(f" ❌ FAIL: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="AIHarness Integration Test Client")
    parser.add_argument("--port", type=int, default=8787, help="HTTP server port")
    parser.add_argument("--project", type=str, default="default", help="Project ID to test against")
    args = parser.parse_args()

    base_url = f"http://127.0.0.1:{args.port}"
    
    # 1. Health Check
    def test_health():
        resp = requests.get(f"{base_url}/")
        resp.raise_for_status()
        if "Running" not in resp.text:
            raise Exception("Server not running correctly")

    if not run_test("Health Check", test_health):
        sys.exit(1)

    # 2. List Tools
    def test_list_tools():
        resp = requests.get(f"{base_url}/tools")
        resp.raise_for_status()
        data = resp.json()
        tools = [t['name'] for t in data['tools']]
        required = ['read_file', 'write_file', 'todo_add', 'build_list_commands']
        for r in required:
            if r not in tools:
                raise Exception(f"Missing tool: {r}")

    if not run_test("List Tools", test_list_tools):
        sys.exit(1)

    # Helper for tool calls
    def call_tool(name, arguments):
        payload = {
            "name": name,
            "arguments": arguments,
            "project_id": args.project
        }
        resp = requests.post(f"{base_url}/call", json=payload)
        resp.raise_for_status()
        data = resp.json()
        if not data.get("success"):
            raise Exception(f"Tool call failed: {data.get('content') or data.get('error')}")
        return data["content"]

    # 3. System Self-Test
    def test_self_test():
        resp_str = call_tool("system_self_test", {"project_path": "/tmp"})
        if "PASS" not in resp_str:
            raise Exception(f"Self-test failed:\n{resp_str}")

    if not run_test("System Self-Test", test_self_test):
        sys.exit(1)

    # 4. File System Tests
    test_file_path = os.path.abspath(f"/tmp/aiharness_test_{int(time.time())}.txt")
    
    def test_file_ops():
        # Write
        content = f"Test content {time.time()}"
        call_tool("write_file", {"path": test_file_path, "content": content})
        
        # Read
        read_back = call_tool("read_file", {"path": test_file_path})
        if read_back != content:
            raise Exception("Read content does not match written content")
            
        # List dir (check if file exists in listing of /tmp)
        listing = call_tool("list_directory", {"path": "/tmp"})
        if os.path.basename(test_file_path) not in listing:
             raise Exception("File not found in directory listing")

    if not run_test("File Operations", test_file_ops):
        sys.exit(1)
        
    # Cleanup file
    try:
        os.remove(test_file_path)
    except:
        pass

    # 4. Todo Tests
    def test_todos():
        # Add
        title = f"Test Todo {int(time.time())}"
        resp_str = call_tool("todo_add", {"title": title, "project_id": args.project})
        todo = json.loads(resp_str)
        todo_id = todo['id']
        
        # List
        list_str = call_tool("todo_list", {"project_id": args.project})
        todos = json.loads(list_str)
        if not any(t['id'] == todo_id for t in todos):
             raise Exception("Added todo not found in list")
             
        # Check
        call_tool("todo_check", {"id": todo_id, "completed": True})
        
        # Verify Check
        list_str_2 = call_tool("todo_list", {"project_id": args.project})
        todos_2 = json.loads(list_str_2)
        target = next(t for t in todos_2 if t['id'] == todo_id)
        if not target['completed']:
            raise Exception("Todo was not marked completed")
            
        # Remove
        call_tool("todo_remove", {"id": todo_id})
        
        # Verify Removal
        list_str_3 = call_tool("todo_list", {"project_id": args.project})
        todos_3 = json.loads(list_str_3)
        if any(t['id'] == todo_id for t in todos_3):
             raise Exception("Todo was not removed")

    if not run_test("Todo Lifecycle", test_todos):
        sys.exit(1)

    print("\nAll integration tests passed!")

if __name__ == "__main__":
    main()
