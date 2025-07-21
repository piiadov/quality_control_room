class WebSocketService {
    constructor(url, timeout = 5000) {
        // Auto-detect protocol based on current page
        this.url = this.adaptUrlProtocol(url);
        this.timeout = timeout;
        this.socket = null;
    }

    adaptUrlProtocol(url) {
        // If running on HTTPS, convert ws:// to wss://
        if (window.location.protocol === 'https:') {
            return url.replace(/^ws:\/\//, 'wss://');
        }
        // If running on HTTP, convert wss:// to ws://
        if (window.location.protocol === 'http:') {
            return url.replace(/^wss:\/\//, 'ws://');
        }
        return url;
    }

    connect() {
        return new Promise((resolve, reject) => {
            console.log(`Attempting to connect to WebSocket: ${this.url}`);
            this.socket = new WebSocket(this.url);

            this.socket.onopen = () => {
                console.log('WebSocket connection opened successfully');
                resolve({ status: 'connected' });
            };

            this.socket.onclose = () => {
                console.log('WebSocket connection closed');
                resolve({ status: 'closed' });
            };

            this.socket.onerror = (error) => {
                console.error('WebSocket connection error:', error);
                reject({ status: 'error', message: 'Backend connection failed' });
            };
        });
    }

    sendCommand(command, store) {
        return new Promise((resolve, reject) => {
            const message = {
                // Structure matched with the backend
                kind: store.kind,
                command: command,
                test_mode: store.testMode,
                population_size: store.batchVolume,
                min_value: store.minValue,
                max_value: store.maxValue,
                data: store.samplingData,
                bins_number: store.binsNumber,
                params_min: store.paramsMin,
                params_max: store.paramsMax,
                predicted_params: store.predictedParams,
                test_mode_params: store.testModeParams,
            };
            // alert("Sending command: " + JSON.stringify(message));
            
            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send(JSON.stringify(message));

                this.socket.onmessage = (event) => {
                    const response = JSON.parse(event.data);
                    console.log('Received response:', response);
                    resolve({ status: 'response received', data: response });
                };

                setTimeout(() => {
                    reject({ status: 'error', message: 'No response from the server within timeout' });
                }, this.timeout);

            } else {
                reject({ status: 'error', message: 'Socket not connected. Command not sent.' });
            }
        });
    }

    closeConnection() {
        if (this.socket) {
            console.log('Closing WebSocket connection...');
            this.socket.close();
            this.socket = null;
        }
    }

    connectAndSendData(command, storage) {
        return this.connect()
            .then(connectionStatus  => {
                console.log('Connection status:', connectionStatus.status);
                if (connectionStatus.status === 'connected') {
                    return this.sendCommand(command, storage);
                } else {
                    throw connectionStatus;
                }
            })
            .then(sendStatusAndResponse => {
                console.log('Send status:', sendStatusAndResponse.status);
                this.closeConnection();
                return sendStatusAndResponse;
            })
            .catch(error => {
                console.error('Error during WebSocket operation:', error.message);
                this.closeConnection();
                throw error;
            });
    }
}

export default WebSocketService;
