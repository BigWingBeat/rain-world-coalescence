using BepInEx;
using multiplayer_mvp.interop;
using multiplayer_mvp.plugin.menu;

namespace multiplayer_mvp.plugin
{
    [BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
    public class MultiplayerMvpPlugin : BaseUnityPlugin
    {
        public const string PLUGIN_GUID = MyPluginInfo.PLUGIN_GUID;
        public const string PLUGIN_NAME = MyPluginInfo.PLUGIN_NAME;
        public const string PLUGIN_VERSION = MyPluginInfo.PLUGIN_VERSION;

#pragma warning disable CS8618 // Instance gets populated in the Awake method, which is called when the mod is loaded
        public static MultiplayerMvpPlugin Instance;
#pragma warning restore CS8618

#pragma warning disable IDE0051 // This type is a monobehaviour, so the Awake method gets called by Unity
        private void Awake()
        {
            Instance = this;

            Logger.LogInfo($"Hello Rain World from {PLUGIN_NAME}!");

            try
            {
                Interop.Point a = Interop.create_point(1, 2);
                Interop.Point b = Interop.create_point(4, 3);
                Interop.Point result = Interop.add_points(a, b);
                Logger.LogInfo($"{a} + {b} = {result}");
            }
            catch (Exception e)
            {
                Logger.LogError(e);
            }

            On.Menu.MainMenu.ctor += MultiplayerLobby.AddMainMenuButton;
            On.ProcessManager.PostSwitchMainProcess += MultiplayerLobby.SwitchMainProcess;
        }
#pragma warning restore IDE0051
    }
}
