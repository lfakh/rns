<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentPosition } from "@tauri-apps/plugin-geolocation";
  import QRCode from "qrcode";
  import { scan } from "@tauri-apps/plugin-barcode-scanner";

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
  let qrCodeDataUrl = $state("");
  let showQR = $state(false);
  let newMessage = $state("");
  let recipient = $state("");
  let fileInput: HTMLInputElement;

  interface Contact {
    identity_hash: string;
    display_name: string;
    status: string;
  }
  let pendingContacts = $state<Contact[]>([]);
  let showPending = $state(false);

  async function loadData() {
    try {
      identity = await invoke("get_identity");
      messages = await invoke("get_messages");
      pendingContacts = await invoke("get_pending_contacts");
      if (identity) {
        qrCodeDataUrl = await QRCode.toDataURL(identity);
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function acceptFriend(hash: string) {
    try {
      await invoke("accept_handshake", { identityHash: hash });
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  async function scanQR() {
    try {
      const result = await scan();
      if (result && result.content) {
        recipient = result.content;
      }
    } catch (e) {
      console.error("Scanning failed:", e);
    }
  }

  async function sendMessage() {
    if (!newMessage) return;
    try {
      await invoke("send_message", { content: newMessage, recipient });
      newMessage = "";
      await loadData();
    } catch (e) {
      console.error(e);
    }
  }

  async function sendLocation() {
    try {
      const pos = await getCurrentPosition();
      const content = `📍 Location: ${pos.coords.latitude.toFixed(6)}, ${pos.coords.longitude.toFixed(6)}`;
      await invoke("send_message", { content, recipient });
      await loadData();
    } catch (e) {
      console.error("Failed to get location:", e);
    }
  }

  async function handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      const file = target.files[0];
      const reader = new FileReader();
      reader.onload = async () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);
        try {
          await invoke("send_image", { imageData: Array.from(uint8Array), recipient });
          await loadData();
        } catch (err) {
          console.error("Failed to send image:", err);
        }
      };
      reader.readAsArrayBuffer(file);
    }
  }

  onMount(() => {
    loadData();
    const unlistenMsg = listen("new-message", () => {
      loadData();
    });
    const unlistenReq = listen("new-friend-request", () => {
      loadData();
    });
    const unlistenAccept = listen("friend-request-accepted", () => {
      loadData();
    });
    return () => {
      unlistenMsg.then(u => u());
      unlistenReq.then(u => u());
      unlistenAccept.then(u => u());
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
        <h1 class="text-2xl font-black tracking-tight text-white uppercase">Bestra</h1>
        <div class="flex items-center space-x-2">
          <p class="text-[10px] font-mono text-white/70 truncate w-32">ID: {identity}</p>
          <button 
            onclick={() => showQR = !showQR}
            class="p-1 bg-white/10 hover:bg-white/20 rounded text-white transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 17h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" /></svg>
          </button>
        </div>
      </div>
      <div class="flex-1 flex justify-end">
        {#if pendingContacts.length > 0}
          <button 
            onclick={() => showPending = !showPending}
            class="relative p-2 bg-yellow-400/20 hover:bg-yellow-400/30 text-yellow-400 rounded-full transition-all pulse-animation"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" /></svg>
            <span class="absolute -top-1 -right-1 bg-red-500 text-white text-[10px] font-bold px-1.5 py-0.5 rounded-full">{pendingContacts.length}</span>
          </button>
        {/if}
      </div>
    </div>
  </div>

  {#if showPending}
    <div class="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-8" onclick={() => showPending = false}>
      <div class="bg-slate-900 p-6 rounded-3xl shadow-2xl w-full max-w-md flex flex-col space-y-4" onclick={(e) => e.stopPropagation()}>
        <h2 class="text-xl font-bold text-white">Friend Requests</h2>
        <div class="space-y-3 max-h-96 overflow-y-auto pr-2">
          {#each pendingContacts as contact}
            <div class="bg-slate-800 p-4 rounded-2xl flex items-center justify-between border border-slate-700">
              <div class="flex-1 min-w-0 mr-4">
                <p class="text-white font-bold truncate">{contact.display_name}</p>
                <p class="text-slate-400 text-xs font-mono truncate">{contact.identity_hash}</p>
              </div>
              <button 
                onclick={() => acceptFriend(contact.identity_hash)}
                class="px-4 py-2 bg-brand-blue text-white text-sm font-bold rounded-xl active:scale-95 transition-all"
              >
                Accept
              </button>
            </div>
          {/each}
        </div>
        <button 
          onclick={() => showPending = false}
          class="w-full py-3 bg-slate-800 text-white font-bold rounded-xl"
        >
          Close
        </button>
      </div>
    </div>
  {/if}

  {#if showQR}
    <div class="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-8" onclick={() => showQR = false}>
      <div class="bg-white p-6 rounded-3xl shadow-2xl flex flex-col items-center space-y-4" onclick={(e) => e.stopPropagation()}>
        <img src={qrCodeDataUrl} alt="My ID QR Code" class="w-64 h-64" />
        <p class="text-slate-900 font-bold text-lg">My Identity QR</p>
        <p class="text-slate-500 text-xs font-mono break-all text-center max-w-[200px]">{identity}</p>
        <button 
          onclick={() => showQR = false}
          class="w-full py-3 bg-slate-900 text-white font-bold rounded-xl"
        >
          Close
        </button>
      </div>
    </div>
  {/if}

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
        onclick={scanQR}
        class="p-2 bg-slate-800 hover:bg-slate-700 rounded-xl text-slate-400 transition-colors"
        title="Scan QR Code"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
      </button>
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

  @keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.05); }
    100% { transform: scale(1); }
  }
  .pulse-animation {
    animation: pulse 2s infinite ease-in-out;
  }
</style>
