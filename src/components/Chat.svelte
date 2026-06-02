<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  interface Message {
    id: String;
    sender: String;
    content: String;
    timestamp: String;
  }

  let messages = $state<Message[]>([]);
  let identity = $state("");
  let newMessage = $state("");
  let recipient = $state("");

  async fn loadData() {
    try {
      identity = await invoke("get_identity");
      messages = await invoke("get_messages");
    } catch (e) {
      console.error(e);
    }
  }

  async fn sendMessage() {
    if (!newMessage) return;
    try {
      await invoke("send_message", { content: newMessage, recipient });
      newMessage = "";
    } catch (e) {
      console.error(e);
    }
  }

  onMount(() => {
    loadData();
    const unlisten = listen("new-message", () => {
      loadData();
    });
    return () => {
      unlisten.then(u => u());
    };
  });
</script>

<div class="flex flex-col h-screen p-4 bg-gray-900 text-white font-sans">
  <div class="mb-4">
    <h1 class="text-2xl font-bold">RNSD Chat</h1>
    <p class="text-sm text-gray-400">Identity: <span class="font-mono">{identity}</span></p>
  </div>

  <div class="flex-1 overflow-y-auto space-y-2 mb-4">
    {#each messages as msg}
      <div class="p-2 rounded bg-gray-800">
        <p class="text-xs text-blue-400">{msg.sender.slice(0, 8)}...</p>
        <p>{msg.content}</p>
        <p class="text-[10px] text-gray-500 text-right">{msg.timestamp}</p>
      </div>
    {/each}
  </div>

  <div class="space-y-2">
    <input 
      bind:value={recipient} 
      placeholder="Recipient Identity Hash" 
      class="w-full p-2 bg-gray-800 border border-gray-700 rounded focus:outline-none focus:border-blue-500"
    />
    <div class="flex space-x-2">
      <input 
        bind:value={newMessage} 
        onkeydown={(e) => e.key === 'Enter' && sendMessage()}
        placeholder="Type a message..." 
        class="flex-1 p-2 bg-gray-800 border border-gray-700 rounded focus:outline-none focus:border-blue-500"
      />
      <button 
        onclick={sendMessage}
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded font-bold"
      >
        Send
      </button>
    </div>
  </div>
</div>

<style>
  /* Tailwind 4 imports are in global.css */
</style>
