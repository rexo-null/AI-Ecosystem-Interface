"""ISKIN CLI Example - Basic agent usage"""

import os
from openhands.sdk import LLM, Agent, Conversation
from openhands.tools.terminal import TerminalTool
from openhands.tools.file_editor import FileEditorTool


def main():
    # LLM configuration
    llm_address = os.getenv("LLM_ADDRESS", "http://localhost:8080")
    
    llm = LLM(
        model="local/llama.cpp",
        base_url=llm_address,
        api_key="not-needed",
    )
    
    # Create agent with tools
    agent = Agent(
        llm=llm,
        tools=[TerminalTool, FileEditorTool],
    )
    
    # Create conversation
    conversation = Conversation(
        agent=agent,
        workspace="./workspace",
    )
    
    # Send prompt
    conversation.send_message("Create a file hello.txt with 'Hello from ISKIN!'")
    conversation.run()
    
    print("\nConversation completed!")


if __name__ == "__main__":
    main()
