using BepInEx;
using BepInEx.Logging;
using MultiplayerMvpClient.NativeInterop;
using MultiplayerMvpClient.Plugin.Menu;
using UnityEngine;

namespace MultiplayerMvpClient.Plugin
{
    [BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
    public class MultiplayerMvpClientPlugin : BaseUnityPlugin
    {
        public const string PLUGIN_GUID = MyPluginInfo.PLUGIN_GUID;
        public const string PLUGIN_NAME = MyPluginInfo.PLUGIN_NAME;
        public const string PLUGIN_VERSION = MyPluginInfo.PLUGIN_VERSION;

#pragma warning disable CS8618 // Instance gets populated in the Awake method, which is called when the mod is loaded
        public static MultiplayerMvpClientPlugin Instance;

        internal static ManualLogSource logger;
#pragma warning restore CS8618

#pragma warning disable IDE0051 // This type is a monobehaviour, so the Awake method gets called by Unity
        private void Awake()
        {
            Instance = this;

            logger = Logger;

            MultiplayerLobby.SetupHooks();

            On.Menu.MainMenu.ExitButtonPressed += (orig, self) => { logger.LogInfo("Exit button pressed!"); orig(self); };
            Application.wantsToQuit += () => { logger.LogInfo("Wants to quit!"); return true; };
            Application.quitting += () => { logger.LogInfo("Quitting!"); DestroyStaticTaskPools(); };
        }
#pragma warning restore IDE0051

        private static void DestroyStaticTaskPools()
        {
            MultiplayerMvpNative.destroy_static_taskpools();
        }
    }
}
