import numpy as np
from PIL import Image
import matplotlib.pyplot as plt
from scipy.ndimage import sobel
from sklearn.mixture import GaussianMixture

with Image.open("funhouse.jpg") as im:
    arr = np.array(im)
print("Loaded image!")

arr = arr.sum(axis=-1)
sobel_magnitude = np.sqrt(sobel(arr, axis=0)**2 + sobel(arr, axis=1)**2)
thresh = np.percentile(sobel_magnitude.flatten(), 95)
sobel_thresh_img = sobel_magnitude > thresh
sobel_thresh = np.stack(np.where(sobel_thresh_img), axis=-1)
print("Extracted Sobel pointcloud!")

fig = plt.figure()
ax0 = fig.add_subplot(1, 2, 1)
ax0.set_xticks([])
ax0.set_yticks([])
ax0.imshow(sobel_thresh_img, cmap='Greys')
ax1 = fig.add_subplot(1, 2, 2)

n_components = 512
gmm = GaussianMixture(n_components=n_components, max_iter=20)
gmm.fit(sobel_thresh)
print("Fit GMM!")

for i in range(n_components):
    cov = gmm.covariances_[i]
    eigvals, eigvecs = np.linalg.eigh(cov)
    if eigvals[0] < eigvals[1]:
        small_eigval = eigvals[0]
        big_eigval = eigvals[1]
        big_eigvec = eigvecs[:, 1]
    else:
        small_eigval = eigvals[1]
        big_eigval = eigvals[0]
        big_eigvec = eigvecs[:, 0]
    if small_eigval < 0.125*big_eigval:
        start = gmm.means_[i] - 10*big_eigvec
        end = gmm.means_[i] + 10*big_eigvec
        ax1.plot([start[0], end[0]], [start[1], end[1]])

ax1.set_xticks([])
ax1.set_yticks([])

plt.savefig("pythontest.png")