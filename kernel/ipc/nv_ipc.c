/*
 * Nuva OS - Inter-Process Communication Implementation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * FFI Compatibility Layer - Not for kernel core use
 *
 * Zero-Copy Inter-Process Communication (FFI wrapper)
 *
 * This module provides C FFI bindings for the Nuva IPC implementation.
 * The kernel core path uses the Rust-native implementation in nvipc/
 * directly. This C file is retained for external C/C++ code only.
 */

#include <stdint.h>
#include <stddef.h>
#include <stdatomic.h>

/* IPC channel states */
typedef enum {
    IPC_STATE_EMPTY = 0,     /* Channel is empty */
    IPC_STATE_SENDING = 1,   /* Message is being sent */
    IPC_STATE_READY = 2,     /* Message is ready to receive */
    IPC_STATE_RECEIVING = 3, /* Message is being received */
} ipc_state_t;

/* IPC channel structure */
typedef struct {
    uint64_t sender;           /* Sender process ID */
    uint64_t receiver;         /* Receiver process ID */
    void *buffer;              /* Message buffer (owned by channel) */
    size_t buffer_size;        /* Buffer size */
    size_t message_size;       /* Actual message size */
    atomic_uint state;         /* Channel state */
    atomic_uint ref_count;     /* Reference count */
    /* Zero-copy support */
    void *external_buffer;     /* External buffer (sender-owned, transferred) */
    int owns_buffer;           /* 1 if channel owns the buffer, 0 if external */
    /* Statistics */
    atomic_ullong messages_sent;
    atomic_ullong messages_received;
    atomic_ullong bytes_sent;
    atomic_ullong bytes_received;
    atomic_ullong send_errors;
    atomic_ullong receive_errors;
} ipc_channel_t;

/* IPC message header */
typedef struct {
    uint64_t sender;           /* Sender process ID */
    uint64_t receiver;         /* Receiver process ID */
    uint32_t message_type;     /* Message type */
    uint32_t flags;            /* Message flags */
    size_t payload_size;       /* Payload size */
} ipc_message_header_t;

/* IPC error codes */
typedef enum {
    IPC_SUCCESS = 0,           /* Success */
    IPC_ERROR_INVALID = -1,    /* Invalid parameter */
    IPC_ERROR_BUSY = -2,       /* Channel is busy */
    IPC_ERROR_EMPTY = -3,      /* Channel is empty */
    IPC_ERROR_FULL = -4,       /* Channel is full */
    IPC_ERROR_TIMEOUT = -5,    /* Operation timed out */
    IPC_ERROR_PERMISSION = -6, /* Permission denied */
} ipc_error_t;

/**
 * Initialize IPC channel
 *
 * @param channel Channel to initialize
 * @param sender Sender process ID
 * @param receiver Receiver process ID
 * @param buffer Message buffer
 * @param buffer_size Buffer size
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_channel_init(ipc_channel_t *channel,
                     uint64_t sender,
                     uint64_t receiver,
                     void *buffer,
                     size_t buffer_size) {
    if (!channel || !buffer || buffer_size == 0) {
        return IPC_ERROR_INVALID;
    }

    channel->sender = sender;
    channel->receiver = receiver;
    channel->buffer = buffer;
    channel->buffer_size = buffer_size;
    channel->message_size = 0;
    atomic_init(&channel->state, IPC_STATE_EMPTY);
    atomic_init(&channel->ref_count, 1);

    return IPC_SUCCESS;
}

/**
 * Send message (copy mode)
 *
 * Copies the message into the channel buffer.
 *
 * @param channel IPC channel
 * @param message Message to send
 * @param size Message size
 * @param timeout_ms Timeout in milliseconds (0 = non-blocking)
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_send(ipc_channel_t *channel,
             void *message,
             size_t size,
             uint32_t timeout_ms) {
    if (!channel || !message || size == 0) {
        return IPC_ERROR_INVALID;
    }

    if (size > channel->buffer_size) {
        atomic_fetch_add(&channel->send_errors, 1);
        return IPC_ERROR_INVALID;
    }

    /* Try to acquire channel for sending */
    uint32_t expected = IPC_STATE_EMPTY;
    if (!atomic_compare_exchange_strong(&channel->state, &expected, IPC_STATE_SENDING)) {
        atomic_fetch_add(&channel->send_errors, 1);
        return IPC_ERROR_BUSY;
    }

    /* Copy message to channel buffer */
    for (size_t i = 0; i < size; i++) {
        ((uint8_t*)channel->buffer)[i] = ((uint8_t*)message)[i];
    }
    channel->message_size = size;
    channel->external_buffer = NULL;
    channel->owns_buffer = 1;

    /* Update statistics */
    atomic_fetch_add(&channel->messages_sent, 1);
    atomic_fetch_add(&channel->bytes_sent, size);

    /* Mark message as ready */
    atomic_store(&channel->state, IPC_STATE_READY);

    return IPC_SUCCESS;
}

/**
 * Send message (zero-copy mode)
 *
 * Transfers ownership of the message buffer without copying.
 * The sender must not access the buffer after this call.
 * The buffer will be returned to the sender via ipc_release().
 *
 * @param channel IPC channel
 * @param message Message buffer (ownership transferred)
 * @param size Message size
 * @param timeout_ms Timeout in milliseconds (0 = non-blocking)
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_send_zero_copy(ipc_channel_t *channel,
                       void *message,
                       size_t size,
                       uint32_t timeout_ms) {
    if (!channel || !message || size == 0) {
        return IPC_ERROR_INVALID;
    }

    /* Try to acquire channel for sending */
    uint32_t expected = IPC_STATE_EMPTY;
    if (!atomic_compare_exchange_strong(&channel->state, &expected, IPC_STATE_SENDING)) {
        atomic_fetch_add(&channel->send_errors, 1);
        return IPC_ERROR_BUSY;
    }

    /* Transfer buffer ownership (zero-copy) */
    channel->external_buffer = message;
    channel->message_size = size;
    channel->owns_buffer = 0;

    /* Update statistics */
    atomic_fetch_add(&channel->messages_sent, 1);
    atomic_fetch_add(&channel->bytes_sent, size);

    /* Mark message as ready */
    atomic_store(&channel->state, IPC_STATE_READY);

    return IPC_SUCCESS;
}

/**
 * Receive message (zero-copy)
 *
 * Returns a pointer to the message buffer without copying.
 * For zero-copy sends, returns the sender's buffer directly.
 * The receiver must call ipc_release() after processing the message.
 *
 * @param channel IPC channel
 * @param message Output pointer to message
 * @param size Output message size
 * @param timeout_ms Timeout in milliseconds (0 = non-blocking)
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_receive(ipc_channel_t *channel,
                void **message,
                size_t *size,
                uint32_t timeout_ms) {
    if (!channel || !message || !size) {
        return IPC_ERROR_INVALID;
    }

    /* Try to acquire channel for receiving */
    uint32_t expected = IPC_STATE_READY;
    if (!atomic_compare_exchange_strong(&channel->state, &expected, IPC_STATE_RECEIVING)) {
        atomic_fetch_add(&channel->receive_errors, 1);
        if (expected == IPC_STATE_EMPTY) {
            return IPC_ERROR_EMPTY;
        } else {
            return IPC_ERROR_BUSY;
        }
    }

    /* Return pointer to message buffer (zero-copy) */
    if (channel->owns_buffer) {
        /* Channel owns the buffer - return internal buffer */
        *message = channel->buffer;
    } else {
        /* External buffer - return sender's buffer directly */
        *message = channel->external_buffer;
    }
    *size = channel->message_size;

    /* Update statistics */
    atomic_fetch_add(&channel->messages_received, 1);
    atomic_fetch_add(&channel->bytes_received, channel->message_size);

    return IPC_SUCCESS;
}

/**
 * Release channel after receiving
 *
 * Must be called after processing a received message.
 *
 * @param channel IPC channel
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_release(ipc_channel_t *channel) {
    if (!channel) {
        return IPC_ERROR_INVALID;
    }

    /* Mark channel as empty */
    atomic_store(&channel->state, IPC_STATE_EMPTY);
    channel->message_size = 0;

    return IPC_SUCCESS;
}

/**
 * Get channel state
 *
 * @param channel IPC channel
 * @return Channel state
 */
ipc_state_t ipc_get_state(ipc_channel_t *channel) {
    if (!channel) {
        return IPC_STATE_EMPTY;
    }
    return (ipc_state_t)atomic_load(&channel->state);
}

/**
 * Check if channel is empty
 *
 * @param channel IPC channel
 * @return 1 if empty, 0 otherwise
 */
int ipc_is_empty(ipc_channel_t *channel) {
    return ipc_get_state(channel) == IPC_STATE_EMPTY;
}

/**
 * Check if channel is ready
 *
 * @param channel IPC channel
 * @return 1 if ready, 0 otherwise
 */
int ipc_is_ready(ipc_channel_t *channel) {
    return ipc_get_state(channel) == IPC_STATE_READY;
}

/**
 * Get channel statistics
 */
typedef struct {
    uint64_t messages_sent;     /* Total messages sent */
    uint64_t messages_received; /* Total messages received */
    uint64_t bytes_sent;        /* Total bytes sent */
    uint64_t bytes_received;    /* Total bytes received */
    uint64_t send_errors;       /* Send errors */
    uint64_t receive_errors;    /* Receive errors */
} ipc_stats_t;

/**
 * Get channel statistics
 *
 * @param channel IPC channel
 * @param stats Output statistics
 * @return IPC_SUCCESS on success, error code otherwise
 */
int ipc_get_stats(ipc_channel_t *channel, ipc_stats_t *stats) {
    if (!channel || !stats) {
        return IPC_ERROR_INVALID;
    }

    /* TODO: Implement statistics tracking */
    stats->messages_sent = 0;
    stats->messages_received = 0;
    stats->bytes_sent = 0;
    stats->bytes_received = 0;
    stats->send_errors = 0;
    stats->receive_errors = 0;

    return IPC_SUCCESS;
}
