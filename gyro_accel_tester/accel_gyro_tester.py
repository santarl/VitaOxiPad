# GamepadIMUviewer v1 by santarl
# Has been expanded by DvaMishkiLapa
# Uses Jibbsmart's JoyShockLibrary

import ctypes
import sys
import time
from collections import deque
from datetime import datetime

import dash
import plotly.graph_objs as go
from dash import dcc, html
from dash.dependencies import Input, Output

deque_maxlen = 40


# Attempt to load the JoyShockLibrary DLL
try:
    joyshock = ctypes.CDLL('./JoyShockLibrary.dll')
except OSError:
    print("\nFailed to load JoyShockLibrary.dll :(")
    print("Please download the DLL from https://github.com/JibbSmart/JoyShockLibrary/releases/download/v3.0/JSL_3_0.zip")
    print("Extract the dll to the same folder as this script and run again\n")
    sys.exit(1)

# Define the function prototypes (adjust these according to the actual DLL)
joyshock.JslConnectDevices.restype = ctypes.c_int
joyshock.JslGetConnectedDeviceHandles.restype = ctypes.c_int
joyshock.JslGetConnectedDeviceHandles.argtypes = [ctypes.POINTER(ctypes.c_int), ctypes.c_int]
joyshock.JslGetAccelX.restype = ctypes.c_float
joyshock.JslGetAccelY.restype = ctypes.c_float
joyshock.JslGetAccelZ.restype = ctypes.c_float
joyshock.JslGetGyroX.restype = ctypes.c_float
joyshock.JslGetGyroY.restype = ctypes.c_float
joyshock.JslGetGyroZ.restype = ctypes.c_float
joyshock.JslGetTimeSinceLastUpdate.restype = ctypes.c_float

# Connecting to devices
result = joyshock.JslConnectDevices()
if result < 0:
    print("Failed to connect to devices.")
    sys.exit(1)

max_devices = 10
device_handles = (ctypes.c_int * max_devices)()
num_devices = joyshock.JslGetConnectedDeviceHandles(device_handles, max_devices)

data_accel = {}
data_gyro = {}
data_ping = {}
colors = ['red', 'green', 'blue', 'purple', 'orange', 'cyan', 'magenta']

for i in range(num_devices):
    device_id = device_handles[i]
    data_accel[device_id] = {'x': deque(maxlen=deque_maxlen), 'y': deque(maxlen=deque_maxlen), 'z': deque(maxlen=deque_maxlen)}
    data_gyro[device_id] = {'x': deque(maxlen=deque_maxlen), 'y': deque(maxlen=deque_maxlen), 'z': deque(maxlen=deque_maxlen)}
    data_ping[device_id] = {'ping': deque(maxlen=deque_maxlen)}

# Initializing the Dash application
app = dash.Dash(__name__)

app.layout = html.Div([
    dcc.Graph(id='accel-graph'),
    dcc.Graph(id='gyro-graph'),
    dcc.Graph(id='ping-graph'),
    dcc.Interval(id='interval-component', interval=100, n_intervals=0)
])


def get_dynamic_range(data, default_min, default_max):
    """
    Function for dynamically changing the axis range if the values are out of range
    """

    min_val = min(min(trace) for trace in data.values() if trace)
    max_val = max(max(trace) for trace in data.values() if trace)

    # Range adaptation when out of range
    if min_val < default_min:
        default_min = min_val - 0.1 * abs(min_val)
    if max_val > default_max:
        default_max = max_val + 0.1 * abs(max_val)

    return default_min, default_max


@app.callback(
    [Output('accel-graph', 'figure'), Output('gyro-graph', 'figure'), Output('ping-graph', 'figure')],
    [Input('interval-component', 'n_intervals')]
)
def update_graph(n):
    accel_traces = []
    gyro_traces = []
    ping_traces = []

    for i in range(num_devices):
        device_id = device_handles[i]
        # Get accelerometer and gyro data
        accel_x = joyshock.JslGetAccelX(device_id)
        accel_y = joyshock.JslGetAccelY(device_id)
        accel_z = joyshock.JslGetAccelZ(device_id)
        gyro_x = joyshock.JslGetGyroX(device_id)
        gyro_y = joyshock.JslGetGyroY(device_id)
        gyro_z = joyshock.JslGetGyroZ(device_id)
        ping = joyshock.JslGetTimeSinceLastUpdate(device_id) * 1000  # sec to msec
        timestamp = time.time()
        formatted_time = datetime.fromtimestamp(timestamp).strftime('%H:%M:%S.%f')

        # Update queues with values
        data_accel[device_id]['x'].append(accel_x)
        data_accel[device_id]['y'].append(accel_y)
        data_accel[device_id]['z'].append(accel_z)
        data_gyro[device_id]['x'].append(gyro_x)
        data_gyro[device_id]['y'].append(gyro_y)
        data_gyro[device_id]['z'].append(gyro_z)
        data_gyro[device_id]['z'].append(gyro_z)
        data_ping[device_id]['ping'].append(ping)

        # Assign color and update legend with current values
        color = colors[i % len(colors)]
        accel_traces.extend([
            go.Scatter(
                y=list(data_accel[device_id]['x']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Accel X: {f"{accel_x:.2f}".rjust(8)}',
                line=dict(color=color, dash='dash')
            ),
            go.Scatter(
                y=list(data_accel[device_id]['y']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Accel Y: {f"{accel_y:.2f}".rjust(8)}',
                line=dict(color=color, dash='dot')
            ),
            go.Scatter(
                y=list(data_accel[device_id]['z']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Accel Z: {f"{accel_z:.2f}".rjust(8)}',
                line=dict(color=color)
            ),
        ])
        gyro_traces.extend([
            go.Scatter(
                y=list(data_gyro[device_id]['x']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Gyro X: {f"{gyro_x:.2f}".rjust(8)}',
                line=dict(color=color, dash='dash')
            ),
            go.Scatter(
                y=list(data_gyro[device_id]['y']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Gyro Y: {f"{gyro_y:.2f}".rjust(8)}',
                line=dict(color=color, dash='dot')
            ),
            go.Scatter(
                y=list(data_gyro[device_id]['z']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Gyro Z: {f"{gyro_z:.2f}".rjust(8)}',
                line=dict(color=color)
            ),
        ])
        ping_traces.extend([
            go.Scatter(
                y=list(data_ping[device_id]['ping']),
                mode='lines',
                legendgrouptitle_text=f"DevID{device_id}: {formatted_time}",
                legendgroup=device_id,
                name=f'Ping (ms): {f"{ping:.2f}".rjust(10)}',
                line=dict(color=color)
            )
        ])

    # Fixed range and adaptation for accelerometer and gyroscope
    accel_min, accel_max = get_dynamic_range(data_accel[device_id], -4, 4)
    gyro_min, gyro_max = get_dynamic_range(data_gyro[device_id], -1000, 1000)
    ping_min, ping_max = get_dynamic_range(data_ping[device_id], 0, 40)

    # Create graphs with dynamic ranges
    accel_fig = go.Figure(data=accel_traces)
    accel_fig.update_layout(
        yaxis=dict(range=[accel_min, accel_max]),
        font=dict(
            family="Courier New, monospace",
            size=15,
            color="RebeccaPurple"
        )
    )
    gyro_fig = go.Figure(data=gyro_traces)
    gyro_fig.update_layout(
        yaxis=dict(range=[gyro_min, gyro_max]),
        font=dict(
            family="Courier New, monospace",
            size=15,
            color="RebeccaPurple"
        )
    )
    ping_fig = go.Figure(data=ping_traces)
    ping_fig.update_layout(
        yaxis=dict(range=[ping_min, ping_max]),
        font=dict(
            family="Courier New, monospace",
            size=15,
            color="RebeccaPurple"
        ),
        showlegend=True
    )

    return accel_fig, gyro_fig, ping_fig


if __name__ == '__main__':
    app.run_server(debug=True)
