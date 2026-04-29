"""ISKIN CLI - Main entry point"""

import os
import sys
import asyncio
from typing import Optional
import argparse

from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.preset.default import get_default_tools


def get_llm_address() -> str:
    """Get LLM server address from user or config"""
    # Try config first
    if os.path.exists("config/llm.toml"):
        try:
            import toml
            with open("config/llm.toml") as f:
                config = toml.load(f)
                return config.get("llm", {}).get("endpoint", "http://localhost:8080")
        except:
            pass
    
    # Ask user
    print("=" * 50)
    print("ISKIN CLI - Local LLM Configuration")
    print("=" * 50)
    print("\nFormats:")
    print("  - http://localhost:8080 (llama.cpp default)")
    print("  - http://localhost:1234 (LM Studio)")
    print()
    addr = input("LLM server address [localhost:8080]: ").strip()
    return addr or "http://localhost:8080"


def create_agent(llm_address: str) -> Agent:
    """Create ISKIN agent with custom tools"""
    llm = LLM(
        model="local/llama.cpp",
        base_url=llm_address,
        api_key="not-needed",  # Local model
    )
    
    tools = get_default_tools()
    
    return Agent(
        llm=llm,
        tools=tools,
    )


def main():
    """Main CLI entry"""
    parser = argparse.ArgumentParser(description="ISKIN CLI - Autonomous AI Agent")
    parser.add_argument("prompt", nargs="*", help="Prompt for the agent")
    parser.add_argument("--interactive", "-i", action="store_true", help="Interactive mode")
    parser.add_argument("--dry-run", action="store_true", help="Simulate without execution")
    
    args = parser.parse_args()
    
    # Get LLM address
    llm_address = get_llm_address()
    
    # Create agent
    agent = create_agent(llm_address)
    workspace = os.getcwd()
    conversation = Conversation(agent=agent, workspace=workspace)
    
    if args.interactive:
        print("ISKIN CLI - Interactive Mode")
        print("Type 'exit' to quit\n")
        while True:
            prompt = input("> ")
            if prompt.lower() in ("exit", "quit"):
                break
            conversation.send_message(prompt)
            conversation.run()
    elif args.prompt:
        prompt = " ".join(args.prompt)
        if args.dry_run:
            prompt = f"[DRY RUN] {prompt}"
        conversation.send_message(prompt)
        conversation.run()
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
