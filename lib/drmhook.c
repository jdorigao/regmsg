/**
 * @file drmhook.c
 * @brief DRM connector mode preference hook
 *
 * This hook intercepts drmModeGetConnector calls to override the preferred display mode
 * based on a configuration file. Useful for forcing specific resolutions/refresh rates
 * on DRM (Direct Rendering Manager) supported systems.
 */

#include <unistd.h> // For close()
#include <stdlib.h> // For NULL
#include <stdio.h> // For FILE, fopen(), fclose(), fgets(), sscanf()
#include <dlfcn.h> // For dlsym(), RTLD_NEXT
#include <stdint.h> // For uint32_t
#include <xf86drm.h> // For DRM file descriptor, etc.
#include <xf86drmMode.h> // For drmModeConnector, drmModeModeInfo, etc.

// Define DRM_MODE_TYPE_PREFERRED if not already defined by the included headers
#ifndef DRM_MODE_TYPE_PREFERRED
#define DRM_MODE_TYPE_PREFERRED 0x1
#endif

/**
 * @typedef drmModeGetConnector_t
 * @brief Function pointer type for the real drmModeGetConnector
 *
 * This typedef defines the signature of the original drmModeGetConnector function
 * that will be dynamically loaded and called.
 */
typedef drmModeConnector *(*drmModeGetConnector_t)(int fd, uint32_t connector_id);

/**
 * @brief Reads preferred mode configuration from a file
 *
 * This function reads a configuration file containing the desired display mode
 * in the format "WIDTHxHEIGHT@REFRESHRATE" (e.g., "1920x1080@60").
 *
 * @param path Path to the configuration file (e.g., "/tmp/drmMode")
 * @param h [out] Pointer to store the width value (horizontal resolution)
 * @param v [out] Pointer to store the height value (vertical resolution)
 * @param r [out] Pointer to store the refresh rate value (in Hz)
 * @return int Returns 1 if the mode is successfully parsed, 0 on failure
 */
int read_preferred_mode(const char *path, uint32_t *h, uint32_t *v, uint32_t *r)
{
    // Attempt to open the configuration file in read-only mode
    FILE *f = fopen(path, "r");
    if (!f)
    {
        // File could not be opened (e.g., does not exist or permission denied)
        return 0;
    }

    // Buffer to store the line read from the file
    char line[128];
    if (fgets(line, sizeof(line), f))
    {
        // Parse the line expecting format "WIDTHxHEIGHT@REFRESHRATE"
        if (sscanf(line, "%ux%u@%u", h, v, r) == 3)
        {
            fclose(f);
            return 1; // Successfully parsed width, height, and refresh rate
        }
    }

    // Close the file if parsing fails or no line is read
    fclose(f);
    return 0; // Indicate failure (e.g., invalid format or empty file)
}

/**
 * @brief Hook function for drmModeGetConnector that overrides preferred mode
 *
 * This function intercepts calls to drmModeGetConnector, retrieves the original
 * connector information, and modifies the mode list to set a user-specified mode
 * as preferred based on the configuration file /tmp/drmMode. It uses dynamic
 * linking to call the original function.
 *
 * @param fd DRM file descriptor for the device
 * @param connector_id Connector ID to query (e.g., HDMI, DisplayPort)
 * @return drmModeConnector* Pointer to the modified connector information,
 *                           or NULL if an error occurs
 */
drmModeConnector *drmModeGetConnector(int fd, uint32_t connector_id)
{
    // Static variable to store the original function pointer, initialized once
    static drmModeGetConnector_t real_drmModeGetConnector = NULL;

    // Load the original drmModeGetConnector function if not already loaded
    if (!real_drmModeGetConnector)
    {
        real_drmModeGetConnector = (drmModeGetConnector_t)dlsym(RTLD_NEXT, "drmModeGetConnector");
        if (!real_drmModeGetConnector)
        {
            // Print error if dynamic linking fails and return NULL
            fprintf(stderr, "Failed to find original drmModeGetConnector: %s\n", dlerror());
            return NULL;
        }
    }

    // Call the original function to get the connector information
    drmModeConnector *connector = real_drmModeGetConnector(fd, connector_id);

    // Check if the connector is valid and has modes available
    if (!connector || connector->count_modes <= 0)
    {
        printf("[hook] No modes found or connector is NULL.\n");
        return connector; // Return unchanged connector (or NULL)
    }

    // Variables to store the preferred mode from the configuration file
    uint32_t pref_width = 0, pref_height = 0, pref_refresh = 0;

    // Attempt to read the preferred mode from the configuration file
    if (!read_preferred_mode("/tmp/drmMode", &pref_width, &pref_height, &pref_refresh))
    {
        printf("[hook] Failed to read /tmp/drmMode, keeping original preferred mode.\n");
        return connector; // Return unchanged if config file read fails
    }

    // Iterate through all modes in the connector's mode list
    for (int i = 0; i < connector->count_modes; i++)
    {
        drmModeModeInfo *mode = &connector->modes[i];

        // Check if the current mode matches the preferred configuration
        if (mode->hdisplay == pref_width &&
            mode->vdisplay == pref_height &&
            mode->vrefresh == pref_refresh)
        {
            // Mark this mode as preferred by setting the flag
            mode->type |= DRM_MODE_TYPE_PREFERRED;
            printf("[hook] Updated preferred mode to: %s %dx%d@%dHz\n",
                   mode->name, mode->hdisplay, mode->vdisplay, mode->vrefresh);
        }
        else
        {
            // Unmark all other modes as preferred by clearing the flag
            mode->type &= ~DRM_MODE_TYPE_PREFERRED;
        }
    }

    // Return the modified connector with the updated preferred mode
    return connector;
}
