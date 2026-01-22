<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useSettingsStore, useBetaStore } from '../store';

const { t } = useI18n();
const settings = useSettingsStore();
const beta = useBetaStore();

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['close']);

const messages = ref([]);
const inputMessage = ref('');
const isLoading = ref(false);
const isConnected = ref(false);
const isAvailable = ref(false);
const chatContainer = ref(null);
const ws = ref(null);
const currentResponse = ref('');

// Engineer WebSocket URL
const engineerUrl = computed(() => {
  const serverUrl = settings.backendUrl || 'wss://quality-control.io:8081/quality';
  // Extract base URL and replace port 8081 with 8082 for engineer service
  try {
    const url = new URL(serverUrl);
    url.port = '8082';
    url.pathname = '/ws';
    return url.toString();
  } catch {
    // Fallback: simple string replacement
    return serverUrl
      .replace(':8081', ':8082')
      .replace('/quality', '/ws')
      .replace('/ws/ws', '/ws');
  }
});

// Build context from current analysis
const analysisContext = computed(() => {
  if (!beta.showResults) return null;
  
  return {
    distribution: beta.distribution === 0 ? 'Beta' : 'Normal',
    sampleSize: beta.sampleSize,
    populationSize: beta.populationSize,
    minValue: beta.minValue,
    maxValue: beta.maxValue,
    params: {
      predicted: beta.predictedParams,
      min: beta.paramsMin,
      max: beta.paramsMax,
      sampling: beta.samplingParams
    },
    chi2: {
      predicted: beta.chi2Pred,
      min: beta.chi2Min,
      max: beta.chi2Max
    },
    testMode: beta.testMode,
    testModeParams: beta.testModeParams
  };
});

const connect = () => {
  if (ws.value?.readyState === WebSocket.OPEN) return;
  
  try {
    ws.value = new WebSocket(engineerUrl.value);
    
    ws.value.onopen = () => {
      isConnected.value = true;
      // Check if Ollama is available
      ws.value.send(JSON.stringify({ command: 'status' }));
    };
    
    ws.value.onclose = () => {
      isConnected.value = false;
      isAvailable.value = false;
    };
    
    ws.value.onerror = () => {
      isConnected.value = false;
      isAvailable.value = false;
    };
    
    ws.value.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      if (data.command === 'status') {
        isAvailable.value = data.available;
        if (!data.available) {
          messages.value.push({
            role: 'system',
            content: 'AI model is not available. Please ensure Ollama is running on the server.'
          });
        }
      } else if (data.command === 'chunk') {
        currentResponse.value += data.content;
        // Update the last assistant message
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.content = currentResponse.value;
        }
        scrollToBottom();
      } else if (data.command === 'done') {
        isLoading.value = false;
        currentResponse.value = '';
        scrollToBottom();
      } else if (data.command === 'error') {
        isLoading.value = false;
        currentResponse.value = '';
        messages.value.push({
          role: 'system',
          content: `Error: ${data.error}`
        });
        scrollToBottom();
      }
    };
  } catch (e) {
    console.error('Failed to connect to Virtual Engineer:', e);
  }
};

const disconnect = () => {
  if (ws.value) {
    ws.value.close();
    ws.value = null;
  }
};

const sendMessage = () => {
  if (!inputMessage.value.trim() || isLoading.value || !isConnected.value) return;
  
  const userMessage = inputMessage.value.trim();
  messages.value.push({ role: 'user', content: userMessage });
  inputMessage.value = '';
  isLoading.value = true;
  currentResponse.value = '';
  
  // Add placeholder for assistant response
  messages.value.push({ role: 'assistant', content: '' });
  
  ws.value.send(JSON.stringify({
    command: 'chat',
    message: userMessage,
    context: analysisContext.value
  }));
  
  scrollToBottom();
};

const scrollToBottom = () => {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
    }
  });
};

const clearChat = () => {
  messages.value = [];
};

// Quick action buttons
const quickActions = [
  { label: 'Explain results', prompt: 'Please explain my analysis results in simple terms.' },
  { label: 'What do params mean?', prompt: 'What do the predicted parameters mean for my quality control?' },
  { label: 'Is the fit good?', prompt: 'Based on the chi-square test results, is the distribution fit good?' },
  { label: 'Suggest actions', prompt: 'Based on these results, what actions should I take?' }
];

const sendQuickAction = (prompt) => {
  inputMessage.value = prompt;
  sendMessage();
};

watch(() => props.visible, (visible) => {
  if (visible) {
    connect();
  }
});

onMounted(() => {
  if (props.visible) {
    connect();
  }
});

onUnmounted(() => {
  disconnect();
});
</script>

<template>
  <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="bg-background border border-borderColor rounded-lg shadow-xl w-full max-w-2xl h-[80vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-borderColor">
        <div class="flex items-center gap-2">
          <i class="fas fa-robot text-xl text-blue-500"></i>
          <h2 class="text-lg font-semibold">Virtual Engineer</h2>
          <span 
            :class="[
              'w-2 h-2 rounded-full',
              isConnected && isAvailable ? 'bg-green-500' : 'bg-red-500'
            ]"
            :title="isConnected && isAvailable ? 'Connected' : 'Disconnected'"
          ></span>
        </div>
        <button @click="emit('close')" class="p-2 hover:bg-backgroundSecondary rounded">
          <i class="fas fa-times"></i>
        </button>
      </div>
      
      <!-- Chat messages -->
      <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-4">
        <!-- Welcome message if no messages -->
        <div v-if="messages.length === 0" class="text-center text-textSecondary py-8">
          <i class="fas fa-robot text-4xl mb-4 text-blue-500/50"></i>
          <p class="mb-4">Hi! I'm the Virtual Engineer. I can help you understand your quality control analysis.</p>
          <p class="text-sm" v-if="analysisContext">I can see your current analysis results and explain them.</p>
          <p class="text-sm" v-else>Run an analysis first, then ask me about the results!</p>
        </div>
        
        <!-- Messages -->
        <div 
          v-for="(msg, idx) in messages" 
          :key="idx"
          :class="[
            'flex',
            msg.role === 'user' ? 'justify-end' : 'justify-start'
          ]"
        >
          <div 
            :class="[
              'max-w-[80%] rounded-lg px-4 py-2',
              msg.role === 'user' 
                ? 'bg-blue-600 text-white' 
                : msg.role === 'system'
                  ? 'bg-yellow-500/20 text-yellow-500 border border-yellow-500/30'
                  : 'bg-backgroundSecondary'
            ]"
          >
            <div class="whitespace-pre-wrap">{{ msg.content }}<span v-if="msg.role === 'assistant' && isLoading && idx === messages.length - 1" class="animate-pulse">▊</span></div>
          </div>
        </div>
      </div>
      
      <!-- Quick actions -->
      <div v-if="messages.length === 0 && analysisContext" class="px-4 pb-2">
        <div class="flex flex-wrap gap-2">
          <button
            v-for="action in quickActions"
            :key="action.label"
            @click="sendQuickAction(action.prompt)"
            :disabled="!isConnected || !isAvailable || isLoading"
            class="px-3 py-1 text-sm bg-backgroundSecondary hover:bg-blue-600/20 rounded-full border border-borderColor disabled:opacity-50"
          >
            {{ action.label }}
          </button>
        </div>
      </div>
      
      <!-- Input area -->
      <div class="p-4 border-t border-borderColor">
        <div class="flex gap-2">
          <input
            v-model="inputMessage"
            @keyup.enter="sendMessage"
            :disabled="!isConnected || !isAvailable || isLoading"
            type="text"
            placeholder="Ask about your analysis..."
            class="flex-1 px-4 py-2 bg-backgroundSecondary border border-borderColor rounded-lg focus:outline-none focus:border-blue-500 disabled:opacity-50"
          />
          <button
            @click="sendMessage"
            :disabled="!isConnected || !isAvailable || isLoading || !inputMessage.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <i class="fas fa-paper-plane"></i>
          </button>
          <button
            @click="clearChat"
            :disabled="messages.length === 0"
            class="px-4 py-2 bg-backgroundSecondary hover:bg-red-600/20 border border-borderColor rounded-lg disabled:opacity-50"
            title="Clear chat"
          >
            <i class="fas fa-trash"></i>
          </button>
        </div>
        <p v-if="!isConnected" class="text-xs text-red-500 mt-2">
          Not connected to Virtual Engineer service
        </p>
        <p v-else-if="!isAvailable" class="text-xs text-yellow-500 mt-2">
          AI model is loading or not available...
        </p>
      </div>
    </div>
  </div>
</template>
