# waybar-groupie

A [waybar custom module](https://github.com/Alexays/Waybar/wiki/Module:-Custom#module-custom-config-return-type) to show Hyprland's grouped window titles.

# Configuration

The configuration is read from a JSON file. Below are the details for each configurable option:

### `separator`
- **Type**: `string`
- **Description**: The string used to separate different elements or sections in the display. This can be customized to fit specific visual or functional requirements.

### `socket_address`
- **Type**: `string`
- **Description**: The address of the socket to which the application will bind. This is crucial for network-related functionalities and ensuring the application can communicate over the specified address. Should be left empty most of the times, the socket address is fetched from the environment.

### `empty_text`
- **Type**: `string`
- **Description**: The text displayed when there is no data or content to show. This can be used to provide user-friendly messages when the application is in an empty state.

### `width`
- **Type**: `int`
- **Description**: The width of the display area in terms of the number of characters or units. Adjusting this value will change the horizontal size of the display.

### `line_height`
- **Type**: `float`
- **Description**: The height of each line in the display. This affects the vertical spacing between lines of text and can be adjusted for better readability or to fit more content on the screen.

### `active_background_color`
- **Type**: `string`
- **Description**: The background color used for active or highlighted elements. This color helps in visually distinguishing active elements from others.

### `background_color`
- **Type**: `string`
- **Description**: The default background color for the display. This sets the overall background color of the application interface.

### Example JSON Configuration

Below is an example of what the JSON configuration file might look like:

```json
{
    "separator": " ||",
    "empty_text": "",
    "width": 100,
    "line_height": 1.0,
    "active_background_color": "#ffffff66",
    "background_color": "#99999966"
}