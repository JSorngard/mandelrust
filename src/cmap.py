import numpy as np
import matplotlib.pyplot as plt

def main():
    ax = plt.figure("3D curve").add_subplot(projection='3d')
    s = np.linspace(0, 1, 1000)
    ax.plot(
        R(s), 
        G(s),
        B(s),
    )
    ax.set_xlim((0, 255))
    ax.set_ylim((0, 255))
    ax.set_zlim((0, 255))

    plt.figure("1D curves")
    plt.plot(R(s), color="red")
    plt.plot(G(s), color="green")
    plt.plot(B(s), color="blue")

    plt.show()
      
def R(s):
    return np.clip(s*255**(1 - 2*s**45), 0, np.inf)

def G(s):
    return np.clip(s*70 - (880*s**18) + (701*s**9), 0, np.inf)

def B(s):
    return np.clip(s*80 + (s**9*255) - (950*s**99), 0, np.inf)

if __name__ == "__main__":
    main()
