using BepInEx;

namespace yourmodname
{
    [BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
    public class YourModName : BaseUnityPlugin
    {
        public const string PLUGIN_GUID = MyPluginInfo.PLUGIN_GUID;
        public const string PLUGIN_NAME = MyPluginInfo.PLUGIN_NAME;
        public const string PLUGIN_VERSION = MyPluginInfo.PLUGIN_VERSION;

        private void Awake()
        {
            Logger.LogInfo($"Hello Rain World from {PLUGIN_NAME}!");
        }
    }
}
