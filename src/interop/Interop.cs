using System.Runtime.InteropServices;
using MonoMod.Utils;
using multiplayer_mvp.plugin;

namespace multiplayer_mvp.interop
{
    public static class Interop
    {
        public const string NATIVE_ASSEMBLY_NAME = "multiplayer_mvp_native";

#pragma warning disable CS8618 // The delegates are initialised by the `ResolveDynDllImports` method via reflection
        static Interop()
        {
            var mapping = new Dictionary<string, List<DynDllMapping>>
            {
                [NATIVE_ASSEMBLY_NAME] = new List<DynDllMapping> {
                    $"{Path.GetDirectoryName(MultiplayerMvpPlugin.Instance.Info.Location)}\\{NATIVE_ASSEMBLY_NAME}.dll"
                }
            };

            typeof(Interop).ResolveDynDllImports(mapping);
        }
#pragma warning restore CS8618

        public delegate Point createPointDelegate(int x, int y);

        public delegate Point addPointsDelegate(Point a, Point b);

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static createPointDelegate create_point;

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static addPointsDelegate add_points;

        [StructLayout(LayoutKind.Sequential)]
        public struct Point
        {
            public int x;
            public int y;
            public Point(int x, int y) { this.x = x; this.y = y; }

            public override readonly string ToString()
            {
                return $"({x}, {y})";
            }
        }
    }
}

