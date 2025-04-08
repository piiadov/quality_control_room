class WebSocketService {
    constructor(url, timeout = 5000) {
        this.url = url;
        this.timeout = timeout;
        this.socket = null;
    }

    connect() {
        return new Promise((resolve, reject) => {
            this.socket = new WebSocket(this.url);

            this.socket.onopen = () => {
                resolve({ status: 'connected' });
            };

            this.socket.onclose = () => {
                resolve({ status: 'closed' });
            };

            this.socket.onerror = () => {
                reject({ status: 'error', message: 'Backend connection failed' });
            };
        });
    }

    sendCommand(command, storage) {
        return new Promise((resolve, reject) => {
            const message = {
                // Structure matched with the backend
                command: command,
                test_mode: storage.testMode,
                population_size: storage.batchVolume,
                min_value: storage.minValue,
                max_value: storage.maxValue,
                data: storage.samplingData,
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
