from math import sin, cos, pi

def solve_nonlin(Γ):
    assert(Γ > -2)

    # We want to solve f(t) = tsint/(cost-1) = Γ

    # We use the approximation f(t) ≈ g(t) = pi^3/2 * 1/(2pi - t) - pi^2/2
    # g(t) = Γ
    # pi^3/2 * 1/(2pi - t) - pi^2/2 = Γ
    # pi^3/2 * 1/(2pi - t) = Γ + pi^2/2
    # 2pi - t = pi^3/(2Γ + pi^2)
    # t = 2*pi - pi^3/(2Γ + pi^2)
    t = 2*pi - pi**3/(2*Γ + pi**2)
    print("t0", t)

    for _ in range(2):
        # newton's method
        f = t * sin(t) / (cos(t)-1) - Γ
        dfdt = (t * sin(t) ** 2 + t * (cos(t) - 1) * cos(t) + sin(t) * (cos(t) - 1)) / (cos(t) - 1) ** 2
        t = t - f / dfdt

    return t

def find_sin(m, k, n, t1):
    Γ = k * t1 / (m - n)
    print(Γ)
    γ = solve_nonlin(Γ)
    ω = γ / t1
    A = (m - n) / (cos(ω * t1) - 1)
    B = n - A
    φ = -ω * t1

    return A, ω, φ, B

# for Γ in [1e-7, 1, 4, 10, 10000]:
#     t = solve_nonlin(Γ)
#     print(Γ, t, t * sin(t) / (cos(t)-1))

# print(find_sin(0.5, 0.999, 2, 3))

print(solve_nonlin(5.628163096847495))
