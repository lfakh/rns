<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentPosition } from "@tauri-apps/plugin-geolocation";

  interface Message {
    id: string;
    sender: string;
    content: string | null;
    msg_type: string;
    attachment_path: string | null;
    timestamp: string;
  }

  let messages = $state<Message[]>([]);
  let identity = $state("");
  let newMessage = $state("");
  let recipient = $state("");
  let fileInput: HTMLInputElement;

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

  async fn sendLocation() {
    try {
      const pos = await getCurrentPosition();
      const content = `📍 Location: ${pos.coords.latitude.toFixed(6)}, ${pos.coords.longitude.toFixed(6)}`;
      await invoke("send_message", { content, recipient });
    } catch (e) {
      console.error("Failed to get location:", e);
    }
  }

  async fn handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      const file = target.files[0];
      const reader = new FileReader();
      reader.onload = async () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);
        try {
          await invoke("send_image", { imageData: Array.from(uint8Array), recipient });
        } catch (err) {
          console.error("Failed to send image:", err);
        }
      };
      reader.readAsArrayBuffer(file);
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

<div class="flex flex-col h-screen bg-slate-950 text-slate-100 font-sans">
  <!-- Header -->
  <div class="p-6 gradient-bg shadow-lg">
    <div class="flex items-center space-x-4">
      <div class="w-12 h-12 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-md overflow-hidden p-1">
        <img src="/bestra-chat.svg" alt="Bestra Logo" class="w-full h-full object-contain" />
      </div>
      <div>
        <h1 class="text-2xl font-black tracking-tight text-white">RNSD CHAT</h1>
        <p class="text-xs font-mono text-white/70 truncate w-48">ID: {identity}</p>
      </div>
    </div>
  </div>

  <!-- Messages Area -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg}
      <div class="flex flex-col {msg.sender === identity ? 'items-end' : 'items-start'}">
        <div class="max-w-[80%] rounded-2xl p-3 shadow-sm {msg.sender === identity ? 'bg-brand-blue text-white rounded-tr-none' : 'bg-slate-800 text-slate-100 rounded-tl-none border border-slate-700'}">
          {#if msg.msg_type === 'text'}
            <p class="text-sm leading-relaxed">{msg.content}</p>
          {:else if msg.msg_type === 'image' && msg.attachment_path}
            <img 
              src={convertFileSrc(msg.attachment_path)} 
              alt="Sent image" 
              class="rounded-lg max-w-full h-auto"
            />
          {:else if msg.msg_type === 'audio'}
            <div class="flex items-center space-x-2">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" /></svg>
              <span class="text-xs italic">Voice Message</span>
            </div>
          {/if}
          <div class="mt-1 flex justify-between items-center space-x-4">
            <span class="text-[10px] opacity-60 font-mono">{msg.sender.slice(0, 6)}</span>
            <span class="text-[10px] opacity-60">{new Date(msg.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}</span>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Input Area -->
  <div class="p-4 bg-slate-900 border-t border-slate-800 space-y-3">
    <div class="flex space-x-2">
      <input 
        bind:value={recipient} 
        placeholder="Recipient Hash" 
        class="flex-1 p-2 bg-slate-800 border border-slate-700 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-brand-blue"
      />
      <button 
        onclick={sendLocation}
        class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl text-slate-400 transition-colors"
        title="Share Location"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
      </button>
      <button 
        onclick={() => fileInput.click()}
        class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl text-slate-400 transition-colors"
        title="Send Image"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
      </button>
      <input type="file" accept="image/*" class="hidden" bind:this={fileInput} onchange={handleFileChange} />
    </div>

    <div class="flex space-x-2">
      <input 
        bind:value={newMessage} 
        onkeydown={(e) => e.key === 'Enter' && sendMessage()}
        placeholder="Message..." 
        class="flex-1 p-3 bg-slate-800 border border-slate-700 rounded-2xl text-sm focus:outline-none focus:ring-2 focus:ring-brand-blue"
      />
      <button 
        onclick={sendMessage}
        class="px-6 py-3 bg-brand-blue hover:bg-blue-600 rounded-2xl font-bold text-white shadow-lg transition-all active:scale-95"
      >
        Send
      </button>
    </div>
  </div>
</div>

<style>
  /* Custom scrollbar for message area */
  .overflow-y-auto::-webkit-scrollbar {
    width: 4px;
  }
  .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }
  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: #334155;
    border-radius: 10px;
  }
</style>
